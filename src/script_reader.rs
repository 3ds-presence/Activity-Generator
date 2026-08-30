// 3DS Presence — Discord Rich Presence for Nintendo 3DS
// Copyright (C) 2026 3DS Presence - LeonLeBreton
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::path::{Path, PathBuf};

use log::warn;

/// Reads game files from the scripts directory.
///
/// Handles two types of files per `title_id`:
/// - `<title_id>/script.lua` — the Lua activity script;
/// - `<title_id>/code.txt` — the 3DS code (RAM addresses) sent to the 3DS.
pub struct ScriptReader {
    script_dir: PathBuf,
}

impl ScriptReader {
    /// Create a new `ScriptReader`.
    ///
    /// `script_dir` — directory containing `<title_id>/script.lua` and
    /// `<title_id>/code.txt` files.
    #[must_use]
    pub fn new(script_dir: &str) -> Self {
        Self {
            script_dir: PathBuf::from(script_dir),
        }
    }

    /// Base directory containing game files.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.script_dir
    }

    /// Read the Lua script for `title_id`.
    ///
    /// Returns `None` if the script does not exist, is unreadable, or if the
    /// path is a path traversal attempt (resolves outside the script directory).
    #[must_use]
    pub fn read_lua_script(&self, title_id: &str) -> Option<String> {
        let script_path = self.lua_script_path(title_id);
        if !self.is_safe_path(&script_path, "lua_path_traversal_blocked") {
            return None;
        }

        match std::fs::read_to_string(&script_path) {
            Ok(content) => Some(content),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => {
                warn!(
                    "evt=lua_script_read_failed path={} error={e}",
                    script_path.display()
                );
                None
            }
        }
    }

    /// Read the addresses file (`<title_id>/code.txt`) for `title_id`.
    ///
    /// Returns `None` if the file does not exist, is unreadable, or if the
    /// path is a path traversal attempt (resolves outside the script directory).
    #[must_use]
    pub fn read_3ds_script(&self, title_id: &str) -> Option<String> {
        let code_path = self.code_path(title_id);
        if !self.is_safe_path(&code_path, "3ds_code_path_traversal_blocked") {
            return None;
        }

        match std::fs::read_to_string(&code_path) {
            Ok(content) => Some(content),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => {
                warn!(
                    "evt=3ds_code_read_failed path={} error={e}",
                    code_path.display()
                );
                None
            }
        }
    }

    /// Path to `<title_id>/script.lua`.
    fn lua_script_path(&self, title_id: &str) -> PathBuf {
        self.script_dir.join(format!("{title_id}/script.lua"))
    }

    /// Path to `<title_id>/code.txt`.
    fn code_path(&self, title_id: &str) -> PathBuf {
        self.script_dir.join(format!("{title_id}/code.txt"))
    }

    /// Check that `path` resolves inside the scripts directory.
    ///
    /// Anti-path-traversal: canonicalize resolves the real path following symlinks.
    /// If the resolved path doesn't start with the intended script directory, block it.
    /// A missing file is a silent `false` (front door, not an attack).
    fn is_safe_path(&self, path: &Path, evt: &str) -> bool {
        let Ok(canonical_base) = std::fs::canonicalize(&self.script_dir) else {
            return false;
        };
        let Ok(canonical_path) = std::fs::canonicalize(path) else {
            return false;
        };
        if canonical_path.starts_with(&canonical_base) {
            true
        } else {
            warn!("evt={evt} path={}", path.display());
            false
        }
    }
}