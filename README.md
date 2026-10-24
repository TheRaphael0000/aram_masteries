> this is some vibe coded bs I did in like 10min cause i wanted to see my masteries in ARAM, just be aware.

# ARAM Masteries

A fast Rust CLI application that queries the League of Legends Client (LCU) API to list all available champions during an ARAM draft (both from the **bench/reroll pool** and from **teammates**), ranked by your **Champion Mastery Score**.

---

## Features

- **Automatic LCU Detection**: Finds the running League Client port and auth token automatically via process inspection and lockfile parsing.
- **Mastery Ranking**: Ranks champions descending by mastery points, level, and season milestones.
- **Clear Identification**: Tags each champion as `[BENCH POOL]`, `[TEAMMATE]`, or `[CURRENT]`.
- **Live Watch Mode (`-w`)**: Polls in real time and automatically re-renders the table whenever someone rolls or trades.
- **Filtering Options**: Supports `--bench-only`, `--team-only`, and `--no-current`.
- **JSON Support**: `--json` flag for scripting, external tools, or piping.

---

## Installation & Running

### Pre-built Binary (Windows)

Download the latest `aram_masteries.exe` or `.zip` release directly from the [GitHub Releases](https://github.com/TheRaphael0000/aram_masteries/releases) page. No Rust toolchain installation required.

### Building from Source

Ensure you have Rust and Cargo installed, and that your League of Legends client is running and logged in.

#### Quick Run
```bash
# Single check
cargo run

# Live watch mode (recommended during game queue / draft)
cargo run -- -w
```

### CLI Options

```text
Options:
  -w, --watch                Watch mode: continuously monitor champion select and update when champions change
  -i, --interval <INTERVAL>  Polling interval in seconds for watch mode [default: 1.5]
      --bench-only           Show only champions from the bench pool
      --team-only            Show only champions held by teammates
      --no-current           Do not show your currently assigned champion
      --json                 Output results in JSON format
      --port <PORT>          Manually specify LCU port
      --auth <AUTH>          Manually specify LCU remoting auth token
  -h, --help                 Print help
```

---

## Architecture & Development

For technical details, endpoint specifications, and guides on how to edit and extend this codebase, see [AGENTS.md](AGENTS.md).
