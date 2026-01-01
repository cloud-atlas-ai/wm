//! Generative LLM extraction from transcript
//!
//! Reads current state + new transcript → LLM generates complete new state
//! Stores raw JSON output - no strict schema parsing

use crate::state;
use chrono::Utc;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::process::{Command, Stdio};

/// Run wm extract
pub fn run(transcript_path: Option<String>, background: bool) -> Result<(), String> {
    if !state::is_initialized() {
        return Err("Not initialized. Run 'wm init' first.".to_string());
    }

    if background {
        return run_background(transcript_path);
    }

    let transcript = find_transcript(transcript_path)?;
    extract_from_transcript(&transcript)
}

/// Run from hook (called by sg)
pub fn run_hook() -> Result<(), String> {
    if !state::is_initialized() {
        return Ok(()); // Silent success
    }

    let transcript = find_transcript(None)?;
    extract_from_transcript(&transcript)
}

/// Fork to background process
fn run_background(transcript_path: Option<String>) -> Result<(), String> {
    let mut args = vec!["extract".to_string()];
    if let Some(path) = transcript_path {
        args.push("--transcript".to_string());
        args.push(path);
    }

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;

    Command::new(exe)
        .args(&args)
        .env("WM_DISABLED", "") // Clear to allow child to run
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn background process: {}", e))?;

    Ok(())
}

/// Find the transcript file
fn find_transcript(explicit_path: Option<String>) -> Result<String, String> {
    if let Some(path) = explicit_path {
        if std::path::Path::new(&path).exists() {
            return Ok(path);
        }
        return Err(format!("Transcript not found: {}", path));
    }

    // Try environment variable
    if let Ok(path) = std::env::var("CLAUDE_TRANSCRIPT_PATH") {
        if std::path::Path::new(&path).exists() {
            return Ok(path);
        }
    }

    // Try to find in ~/.claude/projects/
    if let Some(home) = dirs::home_dir() {
        let claude_dir = home.join(".claude").join("projects");
        if claude_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&claude_dir) {
                let mut transcripts: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().join("transcript.jsonl").exists())
                    .collect();

                transcripts.sort_by_key(|e| {
                    std::fs::metadata(e.path().join("transcript.jsonl"))
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                });

                if let Some(latest) = transcripts.last() {
                    return Ok(latest.path().join("transcript.jsonl").display().to_string());
                }
            }
        }
    }

    Err("Could not find transcript. Use --transcript <path> to specify.".to_string())
}

/// Generative extraction: LLM receives current state + new transcript, returns updated markdown
fn extract_from_transcript(transcript_path: &str) -> Result<(), String> {
    // Read current state markdown (or empty if first run)
    let current_state = std::fs::read_to_string(state::wm_path("state.md")).unwrap_or_default();

    // Read checkpoint for incremental processing
    let checkpoint_pos = read_checkpoint();

    // Read only new transcript content since checkpoint
    let new_transcript = read_transcript_since_position(transcript_path, checkpoint_pos)?;

    if new_transcript.is_empty() {
        println!("No new transcript content to extract from.");
        return Ok(());
    }

    let lines_count = new_transcript.lines().count();

    // Call LLM with current state + new transcript → get updated markdown
    let new_state = call_generative_extraction(&current_state, &new_transcript)?;

    // Write updated state markdown
    std::fs::write(state::wm_path("state.md"), &new_state)
        .map_err(|e| format!("Failed to write state: {}", e))?;

    // Update checkpoint
    let metadata = std::fs::metadata(transcript_path)
        .map_err(|e| format!("Failed to get transcript metadata: {}", e))?;
    write_checkpoint(metadata.len())?;

    println!("State updated ({} new transcript lines processed)", lines_count);

    Ok(())
}

/// Read checkpoint position from checkpoint.json
fn read_checkpoint() -> u64 {
    let path = state::wm_path("checkpoint.json");
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("transcript_position")?.as_u64())
        .unwrap_or(0)
}

/// Write checkpoint position to checkpoint.json
fn write_checkpoint(position: u64) -> Result<(), String> {
    let checkpoint = serde_json::json!({
        "transcript_position": position,
        "last_extraction": Utc::now().to_rfc3339()
    });
    let content = serde_json::to_string_pretty(&checkpoint)
        .map_err(|e| format!("Failed to serialize checkpoint: {}", e))?;
    std::fs::write(state::wm_path("checkpoint.json"), content)
        .map_err(|e| format!("Failed to write checkpoint: {}", e))?;
    Ok(())
}

/// Read transcript content since a given byte position
fn read_transcript_since_position(path: &str, position: u64) -> Result<String, String> {
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("Failed to open transcript: {}", e))?;

    // Seek to position
    file.seek(SeekFrom::Start(position))
        .map_err(|e| format!("Failed to seek transcript: {}", e))?;

    let reader = BufReader::new(file);
    let mut content = String::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
        content.push_str(&line);
        content.push('\n');
    }

    Ok(content)
}

/// Call LLM with generative approach: current state + transcript → updated markdown
fn call_generative_extraction(
    current_state: &str,
    new_transcript: &str,
) -> Result<String, String> {
    use std::io::Write;

    // Prevent recursion
    // SAFETY: Single-threaded, standard pattern for preventing recursive hooks
    unsafe { std::env::set_var("WM_DISABLED", "1") };

    let system_prompt = r#"You are extracting tacit knowledge and metis from an AI coding session.

Metis is practical wisdom—the know-how that comes from experience. Look for:
- Decisions and their rationale
- Constraints discovered through friction
- Patterns in how work gets done
- Facts about the codebase
- Preferences implied by corrections

Accumulate what you learn into the state document. Write naturally in markdown. Include timestamps for recency. Keep everything from the existing state and add what's new. Note conflicts when new info contradicts old."#;

    let message = format!(
        "CURRENT STATE:\n{}\n\nNEW TRANSCRIPT:\n{}\n\nOUTPUT:",
        current_state, new_transcript
    );

    let mut child = Command::new("claude")
        .args(["-p", "--output-format", "json"])
        .arg("--system-prompt")
        .arg(system_prompt)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn claude CLI: {}", e))?;

    // Write message to stdin (avoids OS arg length limits)
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or("Failed to get stdin handle")?;
        stdin
            .write_all(message.as_bytes())
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for claude CLI: {}", e))?;

    // Re-enable WM
    // SAFETY: Single-threaded, restoring previous state
    unsafe { std::env::remove_var("WM_DISABLED") };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "Claude CLI failed (exit {:?}):\nstderr: {}\nstdout: {}",
            output.status.code(),
            stderr,
            stdout
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Extract result text from Claude CLI JSON response
    extract_result_field(&stdout)
}

/// Extract the "result" field from Claude CLI JSON output
fn extract_result_field(response: &str) -> Result<String, String> {
    let cli_response: serde_json::Value = serde_json::from_str(response)
        .map_err(|e| format!("Failed to parse Claude CLI response: {}", e))?;

    cli_response
        .get("result")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| "Claude CLI response missing 'result' field".to_string())
}

