//! Core data types for WM
//!
//! RDF-inspired structure:
//! - Nodes: Vocabulary (domains, layers, libraries)
//! - Items: Knowledge (decisions, constraints, patterns, facts)
//! - Edges: Relationships (applies_to, uses, grounded_in, supersedes)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Item types - what kind of knowledge this represents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Decision,
    Constraint,
    Preference,
    Pattern,
    Fact,
    Definition,
}

/// Item status - confidence level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    Confirmed,  // User confirmed or grounded in repo
    Grounded,   // Evidenced by files/tool output
    Repeated,   // Observed multiple times
    Inferred,   // Derived by LLM, not yet confirmed
    Tentative,  // Single observation, may decay
    Deprecated, // Superseded or explicitly removed
}

/// Edges - typed relationships to nodes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Edges {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applies_to: Vec<String>, // domain:X, layer:Y

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uses: Vec<String>, // library:X, tool:Y

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grounded_in: Vec<String>, // file:X, commit:Y

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supersedes: Vec<String>, // item:X

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflicts_with: Vec<String>, // item:X

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derived_from: Vec<String>, // item:X
}

/// Provenance - where this item came from
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn: Option<u32>,

    pub timestamp: DateTime<Utc>,
}

/// A knowledge item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,

    #[serde(rename = "type")]
    pub item_type: ItemType,

    pub status: ItemStatus,

    pub text: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,

    #[serde(default)]
    pub edges: Edges,

    pub provenance: Provenance,

    #[serde(default)]
    pub strength: f64,

    #[serde(default)]
    pub pinned: bool,

    pub created_at: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
}

/// A vocabulary node (domain, layer, library, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub label: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Vocabulary - all known nodes by category
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Nodes {
    #[serde(default)]
    pub domains: HashMap<String, Node>,

    #[serde(default)]
    pub layers: HashMap<String, Node>,

    #[serde(default)]
    pub libraries: HashMap<String, Node>,

    #[serde(default)]
    pub tools: HashMap<String, Node>,

    #[serde(default)]
    pub files: HashMap<String, Node>,
}

/// A conflict between items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub items: Vec<String>,
    pub reason: String,
    pub surfaced_at: DateTime<Utc>,
}

/// Checkpoint - where we left off
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Checkpoint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_extraction: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript_position: Option<u64>,
}

/// Project state - the main state file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub schema_version: String,
    pub project_id: String,
    pub updated_at: DateTime<Utc>,

    #[serde(default)]
    pub checkpoint: Checkpoint,

    #[serde(default)]
    pub items: Vec<Item>,

    #[serde(default)]
    pub conflicts: Vec<Conflict>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            schema_version: "0.1".to_string(),
            project_id: String::new(),
            updated_at: Utc::now(),
            checkpoint: Checkpoint::default(),
            items: Vec::new(),
            conflicts: Vec::new(),
        }
    }
}

/// Global profile - user preferences across all projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub schema_version: String,

    #[serde(default)]
    pub user: UserInfo,

    #[serde(default)]
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            schema_version: "0.1".to_string(),
            user: UserInfo::default(),
            items: Vec::new(),
        }
    }
}

/// Hook response format (matches Claude Code expectations)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
}

/// Project-level configuration for WM operations
/// Stored in .wm/config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub operations: OperationsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationsConfig {
    #[serde(default = "default_true")]
    pub extract: bool,

    #[serde(default = "default_true")]
    pub compile: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            operations: OperationsConfig::default(),
        }
    }
}

impl Default for OperationsConfig {
    fn default() -> Self {
        Self {
            extract: true,
            compile: true,
        }
    }
}
