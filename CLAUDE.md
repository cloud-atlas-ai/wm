# CLAUDE.md

Working Memory (WM) — automatic tacit knowledge extraction and context management for AI coding assistants.

## Project Overview

WM solves LLM amnesia by automatically extracting tacit knowledge from conversations and surfacing relevant context each turn. It's the "working memory" layer in the Cloud Atlas AI ecosystem.

**Binary:** `wm`

## Build & Test

```bash
cargo build              # Development build
cargo build --release    # Release build
cargo test               # Run tests
```

## Architecture

### Execution Flow

```
[User prompt] → UserPromptSubmit hook → wm compile → inject context
                                                          ↓
                                                [Claude processes]
                                                          ↓
                                                [Stop hook: sg evaluates]
                                                          ↓
                                                [sg calls: wm extract &]
                                                (background, graceful failure)
```

### Two Operations

| Operation | Trigger | LLM? | Purpose |
|-----------|---------|------|---------|
| `compile` | UserPromptSubmit hook | Yes | Filter state for relevance to current intent, inject as context |
| `extract` | Called by sg (or manual) | Yes | Extract knowledge from transcript, update state |

### Composition with Superego

sg and wm are separate tools composing via shell:

```bash
# sg's stop hook calls:
wm extract --session-id "$SESSION_ID" &
# Background, graceful failure, sg doesn't wait
```

No shared state. No coupling. Unix philosophy.

## Storage

```
.wm/
├── state.md              # Accumulated knowledge (markdown, LLM-appended)
├── checkpoint.json       # Extraction position tracking
├── working_set.md        # Last compiled context (global, legacy)
├── hook.log              # Debug log
└── sessions/
    └── <session-id>/
        └── working_set.md   # Per-session compiled context
```

## CLI Commands

```bash
wm init                              # Create .wm/
wm compile [--intent "..."]          # Compile working set
wm extract [--session-id ID]         # Extract from transcript
wm show [state|working]              # Display current state
wm hook compile --session-id ID      # Hook entry (UserPromptSubmit)
wm hook extract --session-id ID      # Hook entry (called by sg)
```

## How Compile Works

1. **Hook fires** on UserPromptSubmit
2. **Read state.md** (accumulated knowledge)
3. **Read intent** from hook JSON (`{"prompt": "user's message"}`)
4. **Call LLM** to filter state for relevance to intent
5. **Return JSON** with `additionalContext` field
6. **Claude Code injects** as `<system-reminder>` block

### Compile Prompt

```
SYSTEM: You are the working memory for an AI coding assistant.
Working memory holds what's relevant RIGHT NOW—not everything known,
just what helps with this specific intent.

Surface: decisions, constraints, patterns, facts that apply.
Be concise. Output only relevant knowledge as markdown.
If nothing is relevant, output nothing.

USER:
ACCUMULATED KNOWLEDGE:
{contents of state.md}

USER'S CURRENT INTENT:
{user's prompt}

RELEVANT KNOWLEDGE:
```

## How Extract Works

1. **Triggered** by sg's stop hook (or manually)
2. **Read transcript** from Claude Code's session file
3. **Call LLM** to extract tacit knowledge
4. **Append to state.md** with timestamp

## Environment Variables

- `WM_DISABLED=1` — Skip all wm operations
- `SUPEREGO_DISABLED=1` — Set by wm during its own LLM calls (prevents sg evaluating wm's internal calls)

## Module Structure

```
src/
├── main.rs          # CLI (clap)
├── init.rs          # Initialize .wm/
├── compile.rs       # Working set compilation (LLM-powered)
├── extract.rs       # Knowledge extraction (LLM-powered)
├── show.rs          # Display commands
├── state.rs         # File I/O for state, working_set
├── types.rs         # Data structures
└── transcript/
    ├── mod.rs
    ├── types.rs     # Transcript entry types
    └── reader.rs    # JSONL parsing
```

## Debugging

```bash
cat .wm/state.md                    # Current accumulated knowledge
cat .wm/working_set.md              # Last compiled context
cat .wm/hook.log                    # Hook execution log
tail -f .wm/hook.log                # Watch hooks fire
```

## Code Style

- Minimal dependencies (no async runtime)
- Graceful failure: hooks never block Claude, return empty on error
- Recursion prevention: set `WM_DISABLED=1` and `SUPEREGO_DISABLED=1` during LLM calls

## Known Issues

Current open issues tracked in beads: `bd list --status open`
