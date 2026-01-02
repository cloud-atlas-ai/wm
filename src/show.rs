//! Display commands for state and working set

use crate::state;

/// Run wm show <what> [--session-id ID]
pub fn run(what: &str, session_id: Option<&str>) -> Result<(), String> {
    match what {
        "state" => show_state(),
        "working" => show_working(session_id),
        "sessions" => show_sessions(),
        _ => Err(format!(
            "Unknown target: {}. Use: state, working, sessions",
            what
        )),
    }
}

fn show_state() -> Result<(), String> {
    if !state::is_initialized() {
        return Err("Not initialized. Run 'wm init' first.".to_string());
    }

    let path = state::wm_path("state.md");
    match std::fs::read_to_string(&path) {
        Ok(content) if content.trim().is_empty() => {
            println!("_No knowledge captured yet. Run 'wm extract' after some conversations._");
            Ok(())
        }
        Ok(content) => {
            println!("{}", content);
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("_No state.md found. Run 'wm init' first._");
            Ok(())
        }
        Err(e) => Err(format!("Failed to read state.md: {}", e)),
    }
}

fn show_working(session_id: Option<&str>) -> Result<(), String> {
    if !state::is_initialized() {
        return Err("Not initialized. Run 'wm init' first.".to_string());
    }

    // Read from session-specific path if provided, otherwise global
    let content = match session_id {
        Some(id) => {
            let path = state::session_dir(id).join("working_set.md");
            std::fs::read_to_string(&path)
        }
        None => state::read_working_set(),
    };

    match content {
        Ok(c) if c.trim().is_empty() => {
            println!("_No working set compiled yet. Run 'wm compile' first._");
            Ok(())
        }
        Ok(c) => {
            println!("{}", c);
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            match session_id {
                Some(id) => println!("_No working set for session {}. Run 'wm hook compile --session-id {}' first._", id, id),
                None => println!("_No working set compiled yet. Run 'wm compile' first._"),
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to read working set: {}", e)),
    }
}

fn show_sessions() -> Result<(), String> {
    if !state::is_initialized() {
        return Err("Not initialized. Run 'wm init' first.".to_string());
    }

    let sessions_dir = state::wm_path("sessions");
    if !sessions_dir.exists() {
        println!("_No sessions yet._");
        return Ok(());
    }

    let entries = std::fs::read_dir(&sessions_dir)
        .map_err(|e| format!("Failed to read sessions directory: {}", e))?;

    let mut sessions: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    if sessions.is_empty() {
        println!("_No sessions yet._");
        return Ok(());
    }

    sessions.sort();
    println!("# Sessions ({})", sessions.len());
    println!();
    for session in sessions {
        let working_set = state::session_dir(&session).join("working_set.md");
        let has_working = working_set.exists();
        let marker = if has_working { "●" } else { "○" };
        println!("{} {}", marker, session);
    }
    println!();
    println!("● = has working_set.md, ○ = no working set");

    Ok(())
}
