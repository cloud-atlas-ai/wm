//! Display commands for state, working set, and nodes

use crate::state;
use crate::types::{Item, ItemStatus};

/// Run wm show <what>
pub fn run(what: &str) -> Result<(), String> {
    match what {
        "state" => show_state(),
        "working" => show_working(),
        "nodes" => show_nodes(),
        "conflicts" => show_conflicts(),
        _ => Err(format!(
            "Unknown target: {}. Use: state, working, nodes, conflicts",
            what
        )),
    }
}

fn show_state() -> Result<(), String> {
    if !state::is_initialized() {
        return Err("Not initialized. Run 'wm init' first.".to_string());
    }

    let state = state::read_state().map_err(|e| format!("Failed to read state: {}", e))?;

    println!("# WM State");
    println!();
    println!("Project ID: {}", state.project_id);
    println!("Updated: {}", state.updated_at);
    println!(
        "Last extraction: {}",
        state
            .checkpoint
            .last_extraction
            .map(|t| t.to_string())
            .unwrap_or_else(|| "never".to_string())
    );
    println!();

    if state.items.is_empty() {
        println!("_No items yet. Run 'wm extract' to populate._");
    } else {
        println!("## Items ({})", state.items.len());
        println!();

        // Group by type
        let mut by_type: std::collections::HashMap<String, Vec<&Item>> =
            std::collections::HashMap::new();

        for item in &state.items {
            if item.status == ItemStatus::Deprecated {
                continue;
            }
            let type_name = format!("{:?}", item.item_type);
            by_type.entry(type_name).or_default().push(item);
        }

        for (type_name, items) in by_type {
            println!("### {} ({})", type_name, items.len());
            for item in items {
                print_item(item);
            }
            println!();
        }
    }

    if !state.conflicts.is_empty() {
        println!("## Conflicts ({})", state.conflicts.len());
        for conflict in &state.conflicts {
            println!("- Items: {:?}", conflict.items);
            println!("  Reason: {}", conflict.reason);
        }
    }

    Ok(())
}

fn show_working() -> Result<(), String> {
    if !state::is_initialized() {
        return Err("Not initialized. Run 'wm init' first.".to_string());
    }

    match state::read_working_set() {
        Ok(content) => {
            println!("{}", content);
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("_No working set compiled yet. Run 'wm compile' first._");
            Ok(())
        }
        Err(e) => Err(format!("Failed to read working set: {}", e)),
    }
}

fn show_nodes() -> Result<(), String> {
    if !state::is_initialized() {
        return Err("Not initialized. Run 'wm init' first.".to_string());
    }

    let nodes = state::read_nodes().map_err(|e| format!("Failed to read nodes: {}", e))?;

    println!("# Vocabulary Nodes");
    println!();

    if !nodes.domains.is_empty() {
        println!("## Domains");
        for (key, node) in &nodes.domains {
            println!("- domain:{} → {}", key, node.label);
        }
        println!();
    }

    if !nodes.layers.is_empty() {
        println!("## Layers");
        for (key, node) in &nodes.layers {
            println!("- layer:{} → {}", key, node.label);
        }
        println!();
    }

    if !nodes.libraries.is_empty() {
        println!("## Libraries");
        for (key, node) in &nodes.libraries {
            println!("- library:{} → {}", key, node.label);
        }
        println!();
    }

    if !nodes.tools.is_empty() {
        println!("## Tools");
        for (key, node) in &nodes.tools {
            println!("- tool:{} → {}", key, node.label);
        }
        println!();
    }

    Ok(())
}

fn show_conflicts() -> Result<(), String> {
    if !state::is_initialized() {
        return Err("Not initialized. Run 'wm init' first.".to_string());
    }

    let state = state::read_state().map_err(|e| format!("Failed to read state: {}", e))?;

    if state.conflicts.is_empty() {
        println!("No conflicts.");
        return Ok(());
    }

    println!("# Conflicts ({})", state.conflicts.len());
    println!();

    for (i, conflict) in state.conflicts.iter().enumerate() {
        println!("## Conflict {}", i + 1);
        println!("Items: {:?}", conflict.items);
        println!("Reason: {}", conflict.reason);
        println!("Surfaced: {}", conflict.surfaced_at);
        println!();
    }

    Ok(())
}

fn print_item(item: &Item) {
    let status_indicator = match item.status {
        ItemStatus::Confirmed => "✓",
        ItemStatus::Grounded => "◉",
        ItemStatus::Repeated => "↻",
        ItemStatus::Inferred => "◌",
        ItemStatus::Tentative => "?",
        ItemStatus::Deprecated => "✗",
    };

    let pinned = if item.pinned { " [pinned]" } else { "" };

    println!(
        "{} {} ({}){} [strength: {:.2}]",
        status_indicator,
        item.text,
        item.id,
        pinned,
        item.strength
    );

    if let Some(ref rationale) = item.rationale {
        println!("  Rationale: {}", rationale);
    }

    if !item.edges.applies_to.is_empty() {
        println!("  Applies to: {}", item.edges.applies_to.join(", "));
    }

    if !item.edges.uses.is_empty() {
        println!("  Uses: {}", item.edges.uses.join(", "));
    }

    if !item.edges.grounded_in.is_empty() {
        println!("  Grounded in: {}", item.edges.grounded_in.join(", "));
    }
}
