# CLAUDE.md

Working Memory (WM) — automatic tacit knowledge extraction and context management for AI coding assistants.

## Project Overview

WM solves LLM amnesia by automatically extracting tacit knowledge from interactions and keeping relevant context top-of-mind. It's the "working memory" layer in the Cloud Atlas AI ecosystem.

**Binary:** `wm`

See `SPEC.md` for full specification.

## Core Concepts

**Working memory (cognitive sense):** The limited-capacity store holding what's relevant *right now* for the current task. Not comprehensive recall—focused retrieval.

**Key insight:** Relationships replace tags. Items connect to domains/layers/libraries through typed edges. Relevance = graph traversal from detected intent.

**Automatic extraction:** LLM-powered from day one. WM observes and learns without user intervention.

## Build & Test Commands

```bash
cargo build              # Development build
cargo build --release    # Release build
cargo test               # Run all tests
cargo run -- <args>      # Run with args (e.g., cargo run -- init)
```

## Architecture

### Execution Flow

```
[User message] → post-submit hook → wm compile → inject working_set.md
                                                        ↓
                                              [Model processes]
                                                        ↓
                                              [Pre-stop: sg evaluates]
                                                        ↓
                                              [sg hook runs: wm extract &]
                                              (background, graceful failure)
```

**Two operations:**
- `wm compile`: Fast (<100ms), synchronous, in post-submit hook
- `wm extract`: LLM-powered, background process, triggered by sg hook (or manually)

### Composition with Superego

**sg and wm are separate tools.** They compose via shell calls:

```bash
# In sg's hook, after clearing:
wm extract --transcript-path "$TRANSCRIPT" &
# Background process, sg doesn't wait
# If wm not installed or fails, sg continues unaffected
```

No shared state. No coupling. Unix philosophy.

### Module Structure (Planned)

```
src/
├── main.rs              # CLI entry point (clap)
├── init.rs              # Initialize .wm/ directory
├── compile.rs           # Working set compilation
├── extract.rs           # LLM-powered extraction
├── state.rs             # State management (read/write/merge)
├── nodes.rs             # Vocabulary management
├── graph.rs             # Graph traversal for relevance
├── intent.rs            # Intent detection from user message
├── format.rs            # Working set markdown formatting
├── profile.rs           # Global profile handling
├── promote.rs           # Promotion workflow
├── claude.rs            # Claude CLI wrapper
└── transcript/          # Transcript parsing
    ├── types.rs
    └── reader.rs
```

### Data Model

**RDF-inspired structure:**
- **Nodes:** Vocabulary (domains, layers, libraries)
- **Items:** Knowledge (decisions, constraints, patterns, facts)
- **Edges:** Relationships (applies_to, uses, grounded_in, supersedes)

```
decision:use-zod ──applies_to──▶ domain:validation
                 ──uses──▶ library:zod
                 ──grounded_in──▶ file:src/api/validators.ts
```

### Storage

```
.wm/
├── state.json       # Items with edges, conflicts
├── nodes.json       # Vocabulary
├── working_set.md   # Generated output
└── config.yaml      # Project settings

~/.wm/
├── profile.json     # Global preferences/constraints
├── nodes.json       # Global vocabulary
├── inbox.jsonl      # Promotion candidates
└── config.toml      # Defaults
```

## Key Design Decisions

### LLM Extraction is Core

Not manual-first. WM automatically extracts tacit knowledge:
- Decisions made in passing
- Preferences implied by corrections
- Patterns observed in behavior
- Constraints discovered through friction

User only intervenes for promotion approval or conflict resolution.

### Relationships Replace Tags

Don't: `tags: ["validation", "api"]`
Do: `edges: { applies_to: ["domain:validation", "layer:api"] }`

Benefits:
- Controlled vocabulary (link to nodes, not strings)
- Graph traversal for relevance
- Conflicts visible (same domain, different choices)
- LLM extracts relationships better than arbitrary tags

### Transcript as Event Log

WM reads Claude Code's transcript. No separate event storage.

### Compile Fast, Extract Slow

- **Compile:** In hook path, <100ms. No LLM. Read state → filter by intent → format.
- **Extract:** Background. LLM-powered. Seconds. Updates state for next compile.

## CLI Commands

```bash
# Core
wm init                          # Create .wm/
wm compile --intent "..."        # Generate working_set.md
wm extract                       # LLM extraction from transcript
wm extract --transcript "..."    # Explicit transcript path

# State
wm show [state|working|nodes]    # Display current state
wm show conflicts                # Show unresolved conflicts
wm forget <item-id>              # Deprecate item
wm resolve <conflict-id>         # Resolve conflict

# Promotion
wm promote <item-id>             # Suggest promotion to global
wm inbox                         # Show pending promotions
wm approve <candidate-id>        # Approve promotion
wm reject <candidate-id>         # Reject

# Global
wm global show                   # Show global profile
wm global edit <item-id>         # Edit global item

# Hooks
wm hook compile                  # Called by post-submit hook
```

## Environment Variables

- `WM_DISABLED=1` — Disable WM entirely
- `WM_DEBUG=1` — Verbose logging

## Hook Setup

**Post-submit hook (wm's own hook):**
```json
{
  "event": "user_prompt_submit",
  "script": "wm hook compile"
}
```

**Extraction trigger (in sg's hook, optional):**
```bash
# sg adds to its post-clear flow:
command -v wm >/dev/null && wm extract --background || true
```

## Code Style

- Never truncate content
- Minimal dependencies (no async runtime, no regex crate)
- Recursion prevention: set `WM_DISABLED=1` during own LLM calls

## Testing Strategy

```bash
cargo test                       # Unit tests
cargo test -- --ignored          # Integration tests
```

Key scenarios:
1. Init creates valid structure
2. Extract produces valid items from transcript
3. Compile filters by intent (graph traversal)
4. Pinned items always included
5. Conflicts detected on merge

## Debugging

```bash
cat .wm/state.json | jq .        # Check state
cat .wm/working_set.md           # Check last working set
WM_DEBUG=1 wm compile            # Verbose output
```

## Version Targets

### v0.1 — Core

- Init, compile, extract
- Graph structure (nodes, items, edges)
- Intent detection and relevance filtering
- Post-submit hook for compile
- LLM extraction from transcript
- Global profile

### v0.2 — Refinement

- Conflict detection and resolution
- Strength decay
- Periodic extraction option

### v0.3 — Promotion

- Promotion workflow (inbox, approve, reject)
- OH MCP connector (guardrails, metis)

### v0.4 — Polish

- Vocabulary management
- Export/import
- Cross-project pattern detection
