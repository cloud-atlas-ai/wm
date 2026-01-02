//! State management - file I/O helpers for .wm/

use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

const WM_DIR: &str = ".wm";
const WORKING_SET_FILE: &str = "working_set.md";
const HOOK_LOG_FILE: &str = "hook.log";

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
