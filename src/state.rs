//! State management - file I/O helpers for .wm/

use crate::types::Config;
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

const WM_DIR: &str = ".wm";
const WORKING_SET_FILE: &str = "working_set.md";
const HOOK_LOG_FILE: &str = "hook.log";
const CONFIG_FILE: &str = "config.toml";

/// Log a message to .wm/hook.log
pub fn log(context: &str, message: &str) {
    let path = wm_path(HOOK_LOG_FILE);
    let timestamp = Local::now().format("%H:%M:%S");
    let line = format!("[{}] [{}] {}\n", timestamp, context, message);

    // Append to log file, ignore errors (logging should never fail the operation)
    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| f.write_all(line.as_bytes()));
}

/// Get the .wm directory path for the current project
/// Uses CLAUDE_PROJECT_DIR if set (from hooks), otherwise falls back to cwd
pub fn wm_dir() -> PathBuf {
    if let Ok(project_dir) = std::env::var("CLAUDE_PROJECT_DIR") {
        PathBuf::from(project_dir).join(WM_DIR)
    } else {
        PathBuf::from(WM_DIR)
    }
}

/// Check if .wm/ exists in current directory
pub fn is_initialized() -> bool {
    wm_dir().exists()
}

/// Get path to a file within .wm/
pub fn wm_path(filename: &str) -> PathBuf {
    wm_dir().join(filename)
}

/// Read the last compiled working set (legacy global path)
pub fn read_working_set() -> io::Result<String> {
    fs::read_to_string(wm_path(WORKING_SET_FILE))
}

/// Write the compiled working set (legacy global path)
pub fn write_working_set(content: &str) -> io::Result<()> {
    fs::write(wm_path(WORKING_SET_FILE), content)
}

/// Get session-specific directory path
pub fn session_dir(session_id: &str) -> PathBuf {
    wm_path(&format!("sessions/{}", session_id))
}

/// Write working set to session-specific path
/// AIDEV-NOTE: Per-session working_set prevents race conditions when
/// multiple sessions compile concurrently in the same project folder.
pub fn write_working_set_for_session(session_id: &str, content: &str) -> io::Result<()> {
    let dir = session_dir(session_id);
    fs::create_dir_all(&dir)?;
    fs::write(dir.join(WORKING_SET_FILE), content)
}

/// Read project-level config, returns default if not found
pub fn read_config() -> Config {
    let path = wm_path(CONFIG_FILE);
    match fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

/// Write project-level config
pub fn write_config(config: &Config) -> io::Result<()> {
    let path = wm_path(CONFIG_FILE);
    let content = toml::to_string_pretty(config)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, content)
}

/// Check if extract operation is enabled
pub fn is_extract_enabled() -> bool {
    read_config().operations.extract
}

/// Check if compile operation is enabled
pub fn is_compile_enabled() -> bool {
    read_config().operations.compile
}
