//! Core data types for WM

use serde::{Deserialize, Serialize};

/// Hook response format (matches Claude Code expectations)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
}
