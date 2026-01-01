#!/bin/bash
# UserPromptSubmit hook for wm
# Surfaces relevant working memory context for the current intent
#
# AIDEV-NOTE: Uses additionalContext to inject relevant knowledge into Claude's context.
# Never blocks - returns empty response on any failure.

# Skip if wm is disabled
if [ "${WM_DISABLED:-}" = "1" ]; then
    exit 0
fi

# Use CLAUDE_PROJECT_DIR if available, otherwise current directory
PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"

# Skip if not initialized (no .wm directory)
if [ ! -d "$PROJECT_DIR/.wm" ]; then
    exit 0
fi

# Skip if wm binary not available
if ! command -v wm &> /dev/null; then
    exit 0
fi

# Capture stdin (JSON from Claude Code)
INPUT=$(cat)

# Extract session_id from hook input
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // ""')

# session_id is required
if [ -z "$SESSION_ID" ]; then
    exit 0
fi

# Change to project directory for wm to find .wm/
cd "$PROJECT_DIR" || exit 0

# Run wm hook compile with session_id, piping stdin for intent
echo "$INPUT" | wm hook compile --session-id "$SESSION_ID" 2>/dev/null || exit 0
