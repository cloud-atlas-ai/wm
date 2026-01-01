//! Initialize .wm/ in current project

use crate::state::{self, wm_dir, wm_path};
use crate::types::{Nodes, State};
use chrono::Utc;
use std::fs;
use std::io;

/// Run wm init
pub fn run() -> Result<(), String> {
    if state::is_initialized() {
        return Err("Already initialized: .wm/ exists".to_string());
    }

    // Create directory structure
    create_directories().map_err(|e| format!("Failed to create directories: {}", e))?;

    // Create initial state
    let state = State {
        schema_version: "0.1".to_string(),
        project_id: state::generate_project_id(),
        updated_at: Utc::now(),
        ..Default::default()
    };
    state::write_state(&state).map_err(|e| format!("Failed to write state: {}", e))?;

    // Create initial nodes
    let nodes = create_default_nodes();
    state::write_nodes(&nodes).map_err(|e| format!("Failed to write nodes: {}", e))?;

    // Create empty working set
    state::write_working_set("# Working Memory\n\n_No items yet._\n")
        .map_err(|e| format!("Failed to write working set: {}", e))?;

    // Create default config
    create_default_config().map_err(|e| format!("Failed to write config: {}", e))?;

    println!("Initialized .wm/ in current directory");
    println!("  state.json      - knowledge graph");
    println!("  nodes.json      - vocabulary");
    println!("  working_set.md  - compiled context (generated each turn)");
    println!("  config.yaml     - settings");

    Ok(())
}

fn create_directories() -> io::Result<()> {
    fs::create_dir_all(wm_dir())?;
    fs::create_dir_all(wm_path("cache"))?;
    Ok(())
}

fn create_default_nodes() -> Nodes {
    use crate::types::Node;
    use std::collections::HashMap;

    let mut domains = HashMap::new();
    domains.insert(
        "auth".to_string(),
        Node {
            label: "Authentication".to_string(),
            description: None,
            url: None,
        },
    );
    domains.insert(
        "validation".to_string(),
        Node {
            label: "Input Validation".to_string(),
            description: None,
            url: None,
        },
    );
    domains.insert(
        "testing".to_string(),
        Node {
            label: "Testing".to_string(),
            description: None,
            url: None,
        },
    );
    domains.insert(
        "error_handling".to_string(),
        Node {
            label: "Error Handling".to_string(),
            description: None,
            url: None,
        },
    );

    let mut layers = HashMap::new();
    layers.insert(
        "api".to_string(),
        Node {
            label: "API Layer".to_string(),
            description: None,
            url: None,
        },
    );
    layers.insert(
        "ui".to_string(),
        Node {
            label: "UI Layer".to_string(),
            description: None,
            url: None,
        },
    );
    layers.insert(
        "db".to_string(),
        Node {
            label: "Database Layer".to_string(),
            description: None,
            url: None,
        },
    );

    Nodes {
        domains,
        layers,
        libraries: HashMap::new(),
        tools: HashMap::new(),
        files: HashMap::new(),
    }
}

fn create_default_config() -> io::Result<()> {
    let config = r#"# WM Configuration

compile:
  max_tokens: 1500
  include_rationale: true

extract:
  # Model for extraction (inherits from claude CLI auth)
  # model: claude-sonnet

# Decay settings (v0.2)
# decay:
#   tentative_ttl_days: 7
#   strength_half_life_days: 14
"#;

    fs::write(wm_path("config.yaml"), config)
}
