# CLAUDE.md — Instructions for Claude Code

Read the gemini-code-assist comment on the GitHub pull request and act on it.

## Project: Copper Hollow

You are building a Rust application that composes MIDI music in folk/indie/alt-country styles. Read the docs/ folder specs before writing code.

## Build

```bash
cargo build --release
cargo test
cargo clippy -- -D warnings
```

## Architecture Rules

- State is `Arc<Mutex<AppState>>` shared between GUI thread and tokio async tasks (bridge, IPC)
- GUI runs on main thread via iced. Tokio runtime runs in a spawned std::thread
- CLI mode: binary detects subcommand via clap, connects to running GUI process over Unix socket at `/tmp/copper-hollow.sock`, sends JSON command, prints JSON response, exits
- If no GUI process is running, CLI operates in headless mode on an ephemeral AppState
- The Bitwig bridge is a TCP server on 127.0.0.1:9876 inside the tokio runtime
- All composition is deterministic given a seed. Same seed + same settings = same output

## Code Style

- No unwrap() in production paths. Use anyhow::Result or explicit error handling
- Prefer strong typing over stringly-typed interfaces. Enums for roles, instruments, parts
- Keep engine/ pure — no IO, no async, no GUI types. Only data in, data out
- serde Serialize/Deserialize on all data types for CLI JSON interchange
- 480 ticks per beat. All rhythmic values in ticks. 4/4 time assumed unless stated

## Composition Quality Bar

The engine must produce MIDI that sounds musical when loaded into a DAW with appropriate instruments. Specifically:

- Rhythm guitar: strum patterns with proper voicing spread, ghost strums, dynamics
- Bass: walking lines that target chord tones on strong beats, approach notes, octave variation  
- Drums: genre-appropriate patterns (NOT programmatic 4-on-floor). Kick/snare interplay, hat dynamics, ghost notes
- Melody: contour with tension/release, targeting chord tones on downbeats, step motion predominant with occasional leaps, rests for breathing
- Counter-melody: harmonically related but rhythmically independent from lead
- Pads: voice-led sustained chords, not just block triads
- ALL tracks: humanized timing (±5-15 ticks), velocity variation, per-part dynamics scaling

## Production Quality Standards

This is a production system that will be sold to demanding customers. No exceptions.

- **No stubs returning success** — If a command exists in `--help`, it either works correctly or returns an error with non-zero exit code. Never return `ok: true` for unimplemented functionality.
- **No mocks in production paths** — Real engine calls, real file I/O, real state management. Mocks are only acceptable in unit tests where they replace external dependencies.
- **No demoware** — Every feature must be full-stack: CLI → engine → output → verified result. Half-implemented features that look good in a demo but don't actually work are not acceptable.
- **No silent failures** — Every error must be surfaced to the user with a clear message and non-zero exit code. Use `anyhow::Result` throughout.
- **Every PR reviewed** — Genuine code review with posted comments before merge. The reviewer must read the diff, check against specs, and post an approval or change request with specific feedback.
- **Integration tests required** — End-to-end tests that verify actual output: MIDI files on disk, valid JSON responses, correct exit codes, deterministic composition.

## Testing

Write unit tests for:
- Scale construction and pitch class math
- Diatonic chord derivation
- Pattern generation determinism (same seed = same output)
- CLI command parsing
- MIDI file export round-trip

Write integration tests for:
- CLI commands produce correct JSON output and exit codes
- `compose` produces a fully-populated song with patterns on all active tracks
- `export-midi` writes a valid MIDI file to disk that can be read back
- Unimplemented commands return non-zero exit code (not fake success)

## File Naming

- Rust modules: snake_case
- Java classes: PascalCase
- MIDI exports: `{song_title}_{timestamp}.mid`

## RALPH — Autonomous Development Loop

This project uses RALPH (Research, Arrange, Loop, Produce, Heal) for autonomous development.

### Architecture
- `ralph.py` — Python orchestrator that runs serial phases in a loop
- `.ralph/status.json` — Current loop state (phase, issue, branch, PR)
- `.ralph/metrics.json` — Cumulative counters (loops, issues, PRs, errors)
- `.ralph/backlog.json` — Issue priority cache
- `.ralph/prompts/` — Worker prompt templates (researcher, planner, orchestrator, worker, reviewer, monitor)

### Phases (serial)
1. **Research** — Web + codebase scan for improvements, dep updates, gaps
2. **Plan** — Create/update GitHub issues from findings
3. **Orchestrate** — Groom backlog, select highest-priority unblocked issue
4. **Work** — Feature branch, implement, commit, push, create PR
5. **Review** — Code review, verify checks, merge or request changes
6. **Monitor** — Health check main branch (compile, test, clippy)
7. Loop back to Research

### Workers
Each phase spawns a single-shot `claude -p` instance with:
- Narrow tool access (`--allowedTools`)
- Turn limits (`--max-turns`)
- Budget caps (`--max-budget-usd`)
- Focused prompt from `.ralph/prompts/`

### Context Loss Between Phases
Every phase runs in its own `claude -p` process. When a phase ends, its entire context window is lost — the next phase starts from zero with no memory of what the previous worker did or discovered. This means **every phase must externalize its valuable output as durable artifacts before it exits.** If a worker learns something, decides something, or builds something but doesn't persist it outside its context window, that work is gone.

Each phase should leave artifacts appropriate to its role:
- **Research** — GitHub issues capturing findings, with enough detail that a future planner can act on them without re-researching
- **Plan** — GitHub issues with clear titles, acceptance criteria, and labels; updated `.ralph/backlog.json` with priority rankings
- **Orchestrate** — Updated `.ralph/status.json` and `.ralph/backlog.json` reflecting the selected issue and reasoning
- **Work** — Committed and pushed code on a feature branch; a pull request with a descriptive body explaining what was done and why
- **Review** — PR comments explaining review decisions; merged PRs or change-request comments with specific actionable feedback
- **Monitor** — GitHub issues for any failures found; updated `.ralph/status.json` if a halt is needed

The rule is simple: **if it's not in a GitHub issue, PR, commit, or status file, it doesn't exist for the next worker.**

### Running
```bash
python3 ralph.py                    # Unlimited loop
python3 ralph.py --max-loops 3      # Run 3 loops
python3 ralph.py --start-phase work # Resume from work phase
python3 ralph.py --dry-run          # Preview without running Claude
python3 ralph.py --status           # Print current state
python3 ralph.py --reset            # Reset counters before starting
```

### Image Generation & Analysis

When you need to generate or analyze images, use the Gemini Nano Banana Pro tool:

```bash
# Generate an image
python3 tools/gemini-image/gemini_image.py generate \
  --prompt "your prompt here" \
  --output path/to/output.png \
  --bg-color "#1a1a2e"

# Analyze an image
python3 tools/gemini-image/gemini_image.py analyze \
  --input path/to/image.png \
  --prompt "describe what you see"

# Edit an image
python3 tools/gemini-image/gemini_image.py edit \
  --input source.png \
  --prompt "make it more rustic" \
  --output edited.png \
  --bg-color "#1a1a2e"
```

- Always use `--bg-color` matching the target web page background so vignette edges blend
- Generated images are automatically post-processed with ImageMagick vignette
- Uses Nano Banana Pro (Gemini 3 Pro Image) — highest quality model available
- Analysis uses Gemini 3.1 Pro — the most powerful model available (March 2026)
- Requires `GEMINI_API_KEY` env var or `~/.config/gemini/api_key` file

### Tagging
- v0.0.1 — First PR merged (scaffolding)
- v0.1.0 — Core types complete
- v0.2.0 — First engine modules
- v0.3.0 — CLI working
- v0.4.0 — GUI working
- v0.5.0 — Full pipeline
