//! Working set compilation
//!
//! Reads state.md + intent → LLM filters for relevance → outputs working_set.md
//! Acts as working memory: surfaces what's relevant RIGHT NOW for the task

use crate::state;
use crate::types::HookResponse;
use std::process::{Command, Stdio};

/// Run wm compile with optional intent
pub fn run(intent: Option<String>) -> Result<(), String> {
    if !state::is_initialized() {
        return Err("Not initialized. Run 'wm init' first.".to_string());
    }

    let state = std::fs::read_to_string(state::wm_path("state.md")).unwrap_or_default();

    if state.trim().is_empty() {
        println!("No knowledge in state.md yet. Run 'wm extract' first.");
        return Ok(());
    }

    let working_set = compile_with_llm(&state, intent.as_deref())?;

    state::write_working_set(&working_set)
        .map_err(|e| format!("Failed to write working set: {}", e))?;

    println!("Compiled working set to .wm/working_set.md");
    Ok(())
}

/// Run from post-submit hook - reads intent from stdin, outputs JSON
/// Never blocks - returns empty response on any failure
pub fn run_hook() -> Result<(), String> {
    if !state::is_initialized() {
        // Silent success if not initialized
        return Ok(());
    }

    let intent = read_hook_intent();
    let state = std::fs::read_to_string(state::wm_path("state.md")).unwrap_or_default();

    // If no state, return empty response
    if state.trim().is_empty() {
        let response = HookResponse {
            additional_context: None,
        };
        let json = serde_json::to_string(&response).map_err(|e| e.to_string())?;
        println!("{}", json);
        return Ok(());
    }

    // Try LLM call, but don't fail the hook if it errors
    let working_set = match compile_with_llm(&state, intent.as_deref()) {
        Ok(ws) => ws,
        Err(_) => String::new(), // Graceful degradation
    };

    // Write to file for debugging
    let _ = state::write_working_set(&working_set);

    // Output hook response
    let response = HookResponse {
        additional_context: if working_set.trim().is_empty() {
            None
        } else {
            Some(working_set)
        },
    };

    let json = serde_json::to_string(&response).map_err(|e| e.to_string())?;
    println!("{}", json);

    Ok(())
}

/// Read intent from hook input (stdin contains JSON with prompt field)
fn read_hook_intent() -> Option<String> {
    use std::io::{self, Read};

    let mut buffer = String::new();
    if io::stdin().read_to_string(&mut buffer).is_ok() && !buffer.trim().is_empty() {
        // Try to parse as JSON hook input (UserPromptSubmit provides 'prompt' field)
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&buffer) {
            if let Some(prompt) = json.get("prompt").and_then(|v| v.as_str()) {
                return Some(prompt.to_string());
            }
        }
        // Fallback: treat raw input as the intent
        Some(buffer.trim().to_string())
    } else {
        None
    }
}

/// Use LLM to filter state for relevance to intent
fn compile_with_llm(state: &str, intent: Option<&str>) -> Result<String, String> {
    use std::io::Write;

    // Prevent recursion
    // SAFETY: Single-threaded, standard pattern for preventing recursive hooks
    unsafe { std::env::set_var("WM_DISABLED", "1") };

    let intent_text = intent.unwrap_or("general coding task");

    let system_prompt = r#"You are the working memory for an AI coding assistant.

Working memory holds what's relevant RIGHT NOW for the task at hand—not everything known, just what helps with this specific intent.

Given accumulated knowledge and the user's current intent, surface what's relevant:
- Decisions that apply to this task
- Constraints that must be respected
- Patterns that should be followed
- Facts the assistant needs to know

Be concise. Output only the relevant knowledge as markdown.
If nothing is relevant, output nothing."#;

    let message = format!(
        "ACCUMULATED KNOWLEDGE:\n{}\n\nUSER'S CURRENT INTENT:\n{}\n\nRELEVANT KNOWLEDGE:",
        state, intent_text
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

    // Write message to stdin
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
        return Err(format!("Claude CLI failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Extract result from Claude CLI JSON response
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
