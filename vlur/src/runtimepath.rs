use std::path::{self, Path};

use log::trace;
use mlua::prelude::*;
use rkyv::{Archive, Deserialize, Serialize};
use walkdir::WalkDir;

use vlur_bridge::OPT_SEP;

const OPT_SEP_LEN: usize = 1;

/// A structure to manage `&runtimepath`.
///
/// ```text
/// &rtp = /foo/bar , /baz/foobar , /foo/bar/after , /baz/foobar/after
///                              ^
///                             `split_pos` is just before the `OPT_SEP`
/// ```
#[derive(Archive, Deserialize, Serialize, Clone, Default)]
#[archive()]
pub struct RuntimePath {
    rtp: String,
    split_pos: usize,
}

impl RuntimePath {
    fn new(rtp: &str) -> Self {
        trace!("parse &runtimepath");
        debug_assert_eq!(OPT_SEP.len_utf8(), OPT_SEP_LEN);

        let mut split_pos = 0;
        for path in rtp.split(OPT_SEP) {
            if path.ends_with("/after") || path.ends_with("\\after") {
                break;
            }
            split_pos += path.len() + OPT_SEP_LEN;
        }
        split_pos = split_pos.saturating_sub(OPT_SEP_LEN);

        Self {
            rtp: rtp.to_string(),
            split_pos,
        }
    }

    pub fn push(&mut self, path: &str, after: bool) {
        if path.is_empty() {
            return;
        }
        if self.rtp.is_empty() {
            self.rtp = path.to_string();
            if !after {
                self.split_pos = path.len();
            }
            return;
        }
        if after {
            self.rtp.push(OPT_SEP);
            self.rtp.push_str(path);
        } else {
            self.rtp.insert(0, OPT_SEP);
            self.rtp.insert_str(0, path);
            self.split_pos += path.len() + OPT_SEP_LEN;
        }
    }

    /// Add the start packages in `&packpath`.
    ///
    /// - `{dir}/start/*`
    /// - `{dir}/start/*/after`
    /// - `{dir}/pack/*/start/*`
    /// - `{dir}/pack/*/start/*/after`
    pub fn push_package(&mut self, dir: &str) {
        let dir = Path::new(dir);
        if !dir.exists() {
            return;
        }

        let entries = WalkDir::new(dir)
            .min_depth(2)
            .max_depth(5)
            .into_iter()
            .filter_entry(|entry| {
                if !entry.file_type().is_dir() {
                    return false;
                }
                let path = entry.path();
                let Some(rel_path) = strip_prefix(path, dir) else {
                    return false;
                };
                Self::package_filter(rel_path)
            });

        for entry in entries {
            let Ok(entry) = entry else {
                continue;
            };
            let Some(fname) = entry.file_name().to_str() else {
                continue;
            };
            let Some(path) = entry.path().to_str() else {
                continue;
            };

            self.push(path, fname == "after");
        }
    }

    fn package_filter(relative_path: &Path) -> bool {
        let components = relative_path.components().filter_map(|component| {
            // Ignore `../`, `C:\\`, ...
            match component {
                path::Component::Normal(s) => Some(s),
                _ => None,
            }
        });
        for (i, c) in components.enumerate() {
            let Some(c) = c.to_str() else {
                return false;
            };
            let ok = match i {
                0 => c == "start" || c == "pack",
                1 => true,
                2 => c == "start" || c == "after",
                3 => true,
                4 => c == "after",
                _ => false,
            };
            if !ok {
                return false;
            }
        }
        true
    }
}

impl std::ops::AddAssign<&RuntimePath> for RuntimePath {
    fn add_assign(&mut self, other: &Self) {
        let len = other.rtp.len();
        if len == 0 {
            return;
        }
        if len == other.split_pos {
            self.rtp.insert(0, OPT_SEP);
            self.rtp.insert_str(0, &other.rtp);

            self.split_pos += len + OPT_SEP_LEN;
        } else {
            let (paths, after_paths) = other.rtp.split_at(other.split_pos + OPT_SEP_LEN);
            self.rtp.insert_str(0, paths);
            self.rtp.push(OPT_SEP);
            self.rtp.push_str(after_paths);

            self.split_pos += other.split_pos + OPT_SEP_LEN;
        }
    }
}

impl<'a> IntoIterator for &'a RuntimePath {
    type Item = &'a str;
    type IntoIter = std::str::Split<'a, char>;

    fn into_iter(self) -> Self::IntoIter {
        self.rtp.as_str().split(OPT_SEP)
    }
}

impl<'lua> FromLua<'lua> for RuntimePath {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua Lua) -> mlua::Result<Self> {
        let mlua::Value::String(lua_string) = value else {
            return Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "string",
                message: Some("&runtimepath".into()),
            });
        };
        let rtp = lua_string.to_str()?;
        Ok(Self::new(rtp))
    }
}

impl<'a, 'lua> IntoLua<'lua> for &'a RuntimePath {
    fn into_lua(self, lua: &'lua Lua) -> mlua::Result<mlua::Value<'lua>> {
        self.rtp.as_str().into_lua(lua)
    }
}

#[cfg(not(windows))]
#[inline]
fn strip_prefix<'a>(path: &'a Path, prefix: &Path) -> Option<&'a Path> {
    path.strip_prefix(prefix).ok()
}

/// Unlike [`std::path::Path::strip_prefix()`], this function ignores
/// whether the path prefix is `verbatim` or not.
#[cfg(windows)]
fn strip_prefix<'a>(path: &'a Path, prefix: &Path) -> Option<&'a Path> {
    use path::{
        Component,
        Prefix::{Disk, VerbatimDisk, VerbatimUNC, UNC},
    };

    let mut path = path.components();
    for prefix in prefix.components() {
        let Some(path) = path.next() else {
            return None;
        };

        let (Component::Prefix(pc1), Component::Prefix(pc2)) = (path, prefix) else {
            if path != prefix {
                return None;
            }
            continue;
        };

        match (pc1.kind(), pc2.kind()) {
            (VerbatimDisk(vd), Disk(d)) | (Disk(d), VerbatimDisk(vd)) => {
                if vd != d {
                    return None;
                }
            }
            (VerbatimUNC(vu1, vu2), UNC(u1, u2))
            | (UNC(u1, u2), VerbatimUNC(vu1, vu2)) => {
                if vu1 != u1 || vu2 != u2 {
                    return None;
                }
            }
            (p1, p2) => {
                if p1 != p2 {
                    return None;
                }
            }
        }
    }

    Some(path.as_path())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let rtp =
            RuntimePath::new("/foo/bar,/baz/foobar,/foo/bar/after,/baz/foobar/after");
        assert_eq!(rtp.split_pos, 20);
    }

    #[test]
    fn new_without_after() {
        let rtp = RuntimePath::new("/foo/bar,/baz/foobar");
        assert_eq!(rtp.split_pos, 20);
    }

    #[test]
    fn new_only_after() {
        let rtp = RuntimePath::new("/foo/bar/after,/baz/foobar/after");
        assert_eq!(rtp.split_pos, 0);
    }

    #[test]
    fn package_filter() {
        assert!(RuntimePath::package_filter(Path::new("start/foo")));
        assert!(RuntimePath::package_filter(Path::new("start/foo/after")));
        assert!(RuntimePath::package_filter(Path::new("pack/pkg/start/plg")));
        assert!(RuntimePath::package_filter(Path::new(
            "pack/pkg/start/plg/after"
        )));
        assert!(!RuntimePath::package_filter(Path::new("opt/foo")));
        assert!(!RuntimePath::package_filter(Path::new("pack/pkg/opt/plg")));
        assert!(!RuntimePath::package_filter(Path::new("query/vim")));
        assert!(!RuntimePath::package_filter(Path::new(
            "pack/pkg/start/plg/after/foo/bar"
        )));
    }

    #[test]
    fn push() {
        let mut rtp = RuntimePath::new("/foo/bar,/foo/bar/after");
        assert_eq!(rtp.split_pos, 8);
        rtp.push("/baz", false);
        rtp.push("/baz/after", true);
        assert_eq!(rtp.rtp.as_str(), "/baz,/foo/bar,/foo/bar/after,/baz/after");
        assert_eq!(rtp.split_pos, 13);

        let mut rtp = RuntimePath::new("");
        rtp.push("/baz", false);
        assert_eq!(rtp.rtp, "/baz");
        assert_eq!(rtp.split_pos, 4);

        let mut rtp = RuntimePath::new("");
        rtp.push("/baz/after", true);
        assert_eq!(rtp.rtp, "/baz/after");
        assert_eq!(rtp.split_pos, 0);
    }

    #[test]
    fn add_assign() {
        let mut rtp = RuntimePath::new("/foo/bar,/foo/bar/after");
        assert_eq!(rtp.split_pos, 8);

        let other = RuntimePath::new("/path");
        assert_eq!(other.split_pos, 5);
        rtp += &other;
        assert_eq!(rtp.rtp.as_str(), "/path,/foo/bar,/foo/bar/after");
        assert_eq!(rtp.split_pos, 14);

        let other = RuntimePath::new("/baz,/baz/after");
        assert_eq!(other.split_pos, 4);
        rtp += &other;
        assert_eq!(
            rtp.rtp.as_str(),
            "/baz,/path,/foo/bar,/foo/bar/after,/baz/after"
        );
        assert_eq!(rtp.split_pos, 19);
    }

    #[cfg(windows)]
    #[test]
    fn strip_prefix_for_windows() {
        assert!(strip_prefix(Path::new("foo"), Path::new("")).is_some());
        assert_eq!(
            strip_prefix(Path::new("/foo/bar/baz"), Path::new("/foo")),
            Some(Path::new("bar/baz"))
        );
        assert_eq!(
            strip_prefix(
                Path::new(r"C:\Users\foo\some/path"),
                Path::new(r"\\?\C:\Users\")
            ),
            Some(Path::new("foo/some/path"))
        );
        assert_eq!(
            strip_prefix(
                Path::new(r"\\?\C:\Users\foo\some\path"),
                Path::new(r"C:/Users/")
            ),
            Some(Path::new("foo/some/path"))
        );
    }
}
