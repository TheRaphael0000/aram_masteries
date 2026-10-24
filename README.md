> this is some vibe coded bs I did in like 1h cause i wanted to see my masteries in ARAM, just be aware.

# ARAM Masteries

A fast Rust CLI application that queries the League of Legends Client (LCU) API to list all available champions during an ARAM draft (both from the **bench/reroll pool** and from **teammates**), ranked by your **Champion Mastery Score**.

---

## Features

- **Automatic LCU Detection & Auto-Reconnect**: Finds the running League Client port and auth token automatically. If League isn't open yet, it waits for it to launch.
- **Mastery Ranking**: Ranks champions descending by mastery points, level, and season milestones.
- **Clear Identification**: Tags each champion as `[BENCH POOL]`, `[TEAMMATE]`, or `[CURRENT]`.
- **Live Watch by Default**: Starts monitoring right away (just double-click the `.exe`!) and automatically updates whenever someone rolls or trades.
- **Filtering Options**: Supports `--bench-only`, `--team-only`, and `--no-current`.
- **JSON Support**: `--json` flag for scripting, external tools, or piping.

---

## Installation & Running

### Pre-built Binary (Windows)

1. Download the latest `aram_masteries.exe` (or zip) from [GitHub Releases](https://github.com/TheRaphael0000/aram_masteries/releases).
2. **Double-click `aram_masteries.exe`**: It will open a terminal window, automatically connect to League (or wait for League to launch), and display your mastery ranking in real time as soon as draft starts!

### Building from Source

Ensure you have Rust and Cargo installed:

```bash
# Live watch mode (default)
cargo run

# Single-shot check and exit
cargo run -- --once
```

### CLI Options

```text
Options:
  -o, --once                 Run once and exit instead of continuous watch mode
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
