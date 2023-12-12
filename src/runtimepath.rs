use std::path::{self, Path};

use mlua::prelude::*;
use rkyv::{Archive, Deserialize, Serialize};
use walkdir::WalkDir;

use crate::nvim::OPT_SEP;

/// A structure to manage `&runtimepath`.
///
/// Usually `after_path` starts with the separator.
///
/// ```text
/// &rtp = /foo/bar , /baz/foobar , /foo/bar/after , /baz/foobar/after
///       |______________________|____________________________________|
///        path                   after_path
/// ```
#[derive(Archive, Deserialize, Serialize, Clone, Default)]
#[archive()]
pub struct RuntimePath {
    path: String,
    after_path: String,
}

impl RuntimePath {
    pub fn new(rtp: &str) -> Self {
        let mut path_len = 0;
        for p in rtp.split(OPT_SEP) {
            if p.ends_with("/after") || p.ends_with("\\after") {
                break;
            }
            path_len += p.len() + 1;
        }
        path_len = path_len.saturating_sub(1);

        let (path, after_path) = rtp.split_at(path_len);
        Self {
            path: path.to_string(),
            after_path: after_path.to_string(),
        }
    }

    pub fn push(&mut self, path: &str, after: bool) {
        if after {
            self.after_path.push(OPT_SEP);
            self.after_path.push_str(path);
        } else {
            self.path.push(OPT_SEP);
            self.path.push_str(path);
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

impl std::fmt::Display for RuntimePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.path)?;
        f.write_str(&self.after_path)?;
        Ok(())
    }
}

impl std::ops::AddAssign<&RuntimePath> for RuntimePath {
    fn add_assign(&mut self, other: &Self) {
        self.path.push(OPT_SEP);
        self.path.push_str(&other.path);
        self.after_path.push_str(&other.after_path);
    }
}

impl<'a> IntoIterator for &'a RuntimePath {
    type Item = &'a str;
    type IntoIter =
        std::iter::Chain<std::str::Split<'a, char>, std::str::Split<'a, char>>;

    fn into_iter(self) -> Self::IntoIter {
        let path = self.path.as_str().split(OPT_SEP);
        let after_path = self.after_path.as_str().split(OPT_SEP);
        path.chain(after_path)
    }
}

impl<'lua> FromLua<'lua> for RuntimePath {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua Lua) -> mlua::Result<Self> {
        let mlua::Value::String(lua_string) = value else {
            return Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "string",
                message: Some("runtimepath".into()),
            });
        };
        let rtp = lua_string.to_str()?;
        Ok(Self::new(rtp))
    }
}

impl<'a, 'lua> IntoLua<'lua> for &'a RuntimePath {
    fn into_lua(self, lua: &'lua Lua) -> mlua::Result<mlua::Value<'lua>> {
        self.to_string().into_lua(lua)
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
        assert_eq!(
            rtp.to_string().as_str(),
            "/foo/bar,/baz/foobar,/foo/bar/after,/baz/foobar/after"
        );
        assert_eq!(rtp.path.as_str(), "/foo/bar,/baz/foobar");
        assert_eq!(rtp.after_path.as_str(), ",/foo/bar/after,/baz/foobar/after");
    }

    #[test]
    fn new_without_after() {
        let rtp = RuntimePath::new("/foo/bar,/baz/foobar");
        assert_eq!(rtp.path.as_str(), "/foo/bar,/baz/foobar");
        assert_eq!(rtp.after_path.as_str(), "");
    }

    #[test]
    fn new_only_after() {
        let rtp = RuntimePath::new("/foo/bar/after,/baz/foobar/after");
        assert_eq!(rtp.path.as_str(), "");
        assert_eq!(rtp.after_path.as_str(), "/foo/bar/after,/baz/foobar/after");
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
        rtp.push("/baz", false);
        rtp.push("/baz/after", true);
        assert_eq!(
            rtp.to_string().as_str(),
            "/foo/bar,/baz,/foo/bar/after,/baz/after"
        );
    }

    #[test]
    fn add_assign() {
        let mut rtp = RuntimePath::new("/foo/bar,/foo/bar/after");
        let other = RuntimePath::new("/baz,/baz/after");
        rtp += &other;
        assert_eq!(
            rtp.to_string().as_str(),
            "/foo/bar,/baz,/foo/bar/after,/baz/after"
        );
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
