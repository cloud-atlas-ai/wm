//! State management - read/write state.json and nodes.json

use crate::types::{Nodes, Profile, State};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

const WM_DIR: &str = ".wm";
const STATE_FILE: &str = "state.json";
const NODES_FILE: &str = "nodes.json";
const WORKING_SET_FILE: &str = "working_set.md";

/// Get the .wm directory path for the current project
pub fn wm_dir() -> PathBuf {
    PathBuf::from(WM_DIR)
}

/// Check if .wm/ exists in current directory
pub fn is_initialized() -> bool {
    wm_dir().exists()
}

/// Get path to a file within .wm/
pub fn wm_path(filename: &str) -> PathBuf {
    wm_dir().join(filename)
}

/// Read project state from .wm/state.json
pub fn read_state() -> io::Result<State> {
    let path = wm_path(STATE_FILE);
    match fs::read_to_string(&path) {
        Ok(content) => {
            serde_json::from_str(&content).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(State::default()),
        Err(e) => Err(e),
    }
}

/// Write project state to .wm/state.json (atomic)
pub fn write_state(state: &State) -> io::Result<()> {
    let path = wm_path(STATE_FILE);
    let tmp_path = wm_path(&format!("{}.tmp", STATE_FILE));

    let content = serde_json::to_string_pretty(state)
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

    // Write to temp file first
    fs::write(&tmp_path, &content)?;

    // Atomic rename
    fs::rename(&tmp_path, &path)?;

    Ok(())
}

/// Read vocabulary nodes from .wm/nodes.json
pub fn read_nodes() -> io::Result<Nodes> {
    let path = wm_path(NODES_FILE);
    match fs::read_to_string(&path) {
        Ok(content) => {
            serde_json::from_str(&content).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Nodes::default()),
        Err(e) => Err(e),
    }
}

/// Write vocabulary nodes to .wm/nodes.json
pub fn write_nodes(nodes: &Nodes) -> io::Result<()> {
    let path = wm_path(NODES_FILE);
    let content =
        serde_json::to_string_pretty(nodes).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
    fs::write(path, content)
}

/// Read the last compiled working set
pub fn read_working_set() -> io::Result<String> {
    fs::read_to_string(wm_path(WORKING_SET_FILE))
}

/// Write the compiled working set
pub fn write_working_set(content: &str) -> io::Result<()> {
    fs::write(wm_path(WORKING_SET_FILE), content)
}

/// Get global WM directory (~/.wm)
pub fn global_wm_dir() -> io::Result<PathBuf> {
    dirs::home_dir()
        .map(|h| h.join(".wm"))
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "Could not determine home directory"))
}

/// Read global profile from ~/.wm/profile.json
pub fn read_profile() -> io::Result<Profile> {
    let path = global_wm_dir()?.join("profile.json");
    match fs::read_to_string(&path) {
        Ok(content) => {
            serde_json::from_str(&content).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Profile::default()),
        Err(e) => Err(e),
    }
}

/// Write global profile to ~/.wm/profile.json
#[allow(dead_code)]
pub fn write_profile(profile: &Profile) -> io::Result<()> {
    let dir = global_wm_dir()?;
    fs::create_dir_all(&dir)?;

    let path = dir.join("profile.json");
    let content = serde_json::to_string_pretty(profile)
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
    fs::write(path, content)
}

/// Generate a project ID from the current directory
pub fn generate_project_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let cwd = std::env::current_dir().unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    cwd.hash(&mut hasher);
    format!("proj_{:x}", hasher.finish())
}
