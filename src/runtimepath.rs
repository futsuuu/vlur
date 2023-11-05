use std::path::Path;

use mlua::{FromLua, IntoLua, Lua};
use walkdir::WalkDir;

use crate::OPT_SEP;

/// A structure to manage `&runtimepath`.
///
/// Usually `after_path` starts with the separator.
///
/// ```text
/// &rtp = /foo/bar , /baz/foobar , /foo/bar/after , /baz/foobar/after
///       |______________________|____________________________________|
///        path                   after_path
/// ```
#[derive(Clone)]
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
        if path_len != 0 {
            path_len -= 1;
        }

        let (path, after_path) = rtp.split_at(path_len);
        Self {
            path: path.to_string(),
            after_path: after_path.to_string(),
        }
    }

    pub fn add(&mut self, path: &str, after: bool) {
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
    pub fn add_package(&mut self, dir: &str) {
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
                let Some(fname) = entry.file_name().to_str() else {
                    return false;
                };
                match entry.depth() {
                    1 => fname == "start" || fname == "pack",
                    3 => fname == "start" || fname == "after",
                    5 => fname == "after",
                    _ => true,
                }
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

            self.add(path, fname == "after");
        }
    }
}

impl std::fmt::Display for RuntimePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.path)?;
        f.write_str(&self.after_path)?;
        Ok(())
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

impl<'lua> IntoLua<'lua> for RuntimePath {
    fn into_lua(self, lua: &'lua Lua) -> mlua::Result<mlua::Value<'lua>> {
        let rtp = self.path + &self.after_path;
        rtp.into_lua(lua)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let rtp =
            RuntimePath::new("/foo/bar,/baz/foobar,/foo/bar/after,/baz/foobar/after");
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
}