# ARAM Masteries - Developer & Agent Guide (`AGENTS.md`)

This document contains everything an AI agent or human developer needs to understand, maintain, edit, and extend the **`aram_masteries`** application.

---

## 1. Project Overview

`aram_masteries` is a fast, lightweight Rust CLI tool that interfaces with the local League of Legends Client Update (LCU) API. During an ARAM draft (champion select), it:

1. Detects available champions in the **Bench Pool** (rerolled champions) and from **Teammates**.
2. Cross-references the local player's **Champion Mastery** data.
3. Ranks all available champions in descending order of mastery score.
4. Renders a terminal table or outputs JSON.
5. In `--watch` (`-w`) mode, monitors the draft in real time and automatically updates whenever someone rolls or trades.

---

## 2. Codebase Structure

```text
aram_masteries/
├── Cargo.toml               # Dependencies (reqwest, serde, clap, comfy-table, colored, sysinfo)
├── AGENTS.md                # This developer & agent guide
├── README.md                # User-facing documentation
├── src/
│   ├── lib.rs               # Library root: extraction logic, sorting, table formatting, signatures
│   ├── main.rs              # CLI entry point: Clap argument parsing, single-shot & watch loop
│   ├── lcu.rs               # LCU credential discovery, HTTP client, and LCU endpoints
│   └── models.rs            # Serde data transfer objects for LCU JSON responses
└── tests/
    └── draft_test.rs        # Automated test suite for sorting, filtering, and number formatting
```

### Module Responsibilities

- **`src/models.rs`**:
    - `ChampSelectSession`: Represents `/lol-champ-select/v1/session` (`benchChampions`, `myTeam`, `localPlayerCellId`).
    - `ChampionMasteryItem`: Represents `/lol-champion-mastery/v1/local-player/champion-mastery` (`championId`, `championLevel`, `championPoints`, `championPointsUntilNextLevel`, `highestGrade`).
    - `ChampionSummaryItem`: Represents `/lol-game-data/assets/v1/champion-summary.json` (`id`, `name`, `alias`).
    - `AvailableChampion`: Unified structure representing a selectable champion with source (`Current`, `Bench`, `Teammate`).

- **`src/lcu.rs`**:
    - `LcuClient`: Configures HTTP basic auth (`riot:<auth_token>`) and bypasses self-signed TLS certificates (`danger_accept_invalid_certs(true)`).
    - `detect_credentials()`: Discovers LCU credentials via:
        1. Inspecting running processes (`LeagueClientUx.exe`, `LeagueClient.exe`) via `sysinfo` to find the install directory.
        2. Reading the `lockfile` located in the League install folder (format: `process_name:pid:port:token:protocol`).
        3. Falling back to common install paths (`C:\Riot Games\League of Legends\lockfile`, etc.).

- **`src/lib.rs`**:
    - `extract_available_champions()`: Filters, maps, and sorts candidates descending by `mastery_points`, then `mastery_level`, then `name`.
    - `print_champions_table()`: Formats and outputs the UTF-8 border table using `comfy-table`.
    - `generate_signature()`: Generates a hash/signature of the current bench & team picks to avoid re-rendering unless the draft state actually changes.

- **`src/main.rs`**:
    - CLI argument definition (`Cli` struct with `clap`).
    - `run_single()`: Queries current state once and exits.
    - `run_watch()`: Continuous polling loop with ANSI clear and auto-refresh on changes.

---

## 3. LCU API Reference

All requests to the LCU must:

- Use `https://127.0.0.1:<port>`
- Use HTTP Basic Auth: Username `riot`, Password `<remoting-auth-token>`
- Ignore self-signed TLS certificates

### Key Endpoints Used

| Endpoint                                                 | Method | Purpose                                                                         |
| :------------------------------------------------------- | :----- | :------------------------------------------------------------------------------ |
| `/lol-champ-select/v1/session`                           | `GET`  | Returns draft state. If not in champion select, returns `404 Not Found`.        |
| `/lol-champion-mastery/v1/local-player/champion-mastery` | `GET`  | Returns an array of mastery entries for all champions played by the local user. |
| `/lol-game-data/assets/v1/champion-summary.json`         | `GET`  | Fast local lookup of all champion IDs to names.                                 |
| `/lol-summoner/v1/current-summoner`                      | `GET`  | Returns info about the logged-in summoner (name, level, reroll points).         |

---

## 4. How to Build, Test, and Run

### Building

```powershell
# Development build
cargo check
cargo build

# Optimized release binary
cargo build --release
# Executable at: target/release/aram_masteries.exe
```

### Running Tests

```powershell
cargo test
```

### Running the CLI

```powershell
# Single-shot execution
cargo run

# Watch mode (automatically updates during draft countdown)
cargo run -- --watch

# Custom poll interval (e.g. 1.0 second)
cargo run -- -w -i 1.0

# Bench only (filter out teammates)
cargo run -- --bench-only

# Teammates only (filter out bench)
cargo run -- --team-only

# Machine-readable JSON output
cargo run -- --json

# Manual credentials (if needed)
cargo run -- --port 30368 --auth gYRJxzpFqvAVqroVey314A
```

---

## 5. Potential Future Extensions & How to Implement

### 1. Auto-Swap / Quick-Pick from Bench

- **LCU Endpoint**: `POST /lol-champ-select/v1/session/bench/swap/{championId}`
- **How to add**: Add a command or keypress listener (e.g., press `1` to swap for the top mastery champion on the bench).

### 2. Teammate Trade Requests

- **LCU Endpoint**:
    - `GET /lol-champ-select/v1/session/trades` (get tradeable states)
    - `POST /lol-champ-select/v1/session/trades/{id}/request`
- **How to add**: Display trade availability flags and permit requesting trades.

### 3. ARAM Win Rates / Tier Integration

- Query local cache or an external source (such as `Lolalytics` or `Metasrc` ARAM tier list) and add a `Win Rate %` / `Tier` column alongside Mastery Score.

### 4. Reroll Points & Remaining Rolls

- Endpoint `/lol-summoner/v1/current-summoner` contains `rerollPoints.numberOfRolls` and `rerollPoints.pointsCostToRoll`.
- Can be displayed in the table header so the user knows if they have 1 or 2 rerolls available.
