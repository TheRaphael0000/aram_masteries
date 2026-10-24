use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use clap::Parser;
use colored::*;

use aram_masteries::lcu::LcuClient;
use aram_masteries::models::ChampionMasteryItem;
use aram_masteries::{
    extract_available_champions, generate_signature, print_champions_table, ExtractOptions,
};

#[derive(Parser, Debug)]
#[command(name = "aram_masteries")]
#[command(about = "List ARAM draft available champions ordered by mastery score via LCU API")]
struct Cli {
    /// Run once and exit instead of continuous watch mode
    #[arg(short, long)]
    once: bool,

    /// Kept for backwards compatibility (watch mode is now enabled by default)
    #[arg(short, long, hide = true)]
    watch: bool,

    /// Polling interval in seconds for watch mode
    #[arg(short, long, default_value_t = 1.5)]
    interval: f32,

    /// Show only champions from the bench pool
    #[arg(long)]
    bench_only: bool,

    /// Show only champions held by teammates
    #[arg(long)]
    team_only: bool,

    /// Do not show your currently assigned champion
    #[arg(long)]
    no_current: bool,

    /// Output results in JSON format
    #[arg(long)]
    json: bool,

    /// Manually specify LCU port
    #[arg(long)]
    port: Option<u16>,

    /// Manually specify LCU remoting auth token
    #[arg(long)]
    auth: Option<String>,
}

impl From<&Cli> for ExtractOptions {
    fn from(cli: &Cli) -> Self {
        Self {
            bench_only: cli.bench_only,
            team_only: cli.team_only,
            no_current: cli.no_current,
        }
    }
}

fn chrono_now() -> String {
    use std::time::SystemTime;
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = dur.as_secs();
    let secs = total_secs % 60;
    let mins = (total_secs / 60) % 60;
    let hours = (total_secs / 3600) % 24;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

fn wait_for_lcu(
    port: Option<u16>,
    auth: Option<String>,
    interval_secs: f32,
) -> (LcuClient, HashMap<i32, String>) {
    let mut logged_waiting = false;
    let poll_delay = Duration::from_secs_f32(interval_secs.max(1.0));

    loop {
        match LcuClient::new(port, auth.clone()) {
            Ok(client) => {
                println!(
                    "{} Connected to League Client (LCU) on port {}",
                    "✓".green().bold(),
                    client.creds.port.to_string().cyan()
                );
                let champion_names = match client.get_champion_map() {
                    Ok(names) => names,
                    Err(e) => {
                        eprintln!(
                            "{} Failed to load champion summary: {}",
                            "Warning:".yellow().bold(),
                            e
                        );
                        HashMap::new()
                    }
                };
                return (client, champion_names);
            }
            Err(_) => {
                if !logged_waiting {
                    println!(
                        "{} Waiting for League of Legends client to launch and log in...",
                        "⏳".bold()
                    );
                    logged_waiting = true;
                }
                thread::sleep(poll_delay);
            }
        }
    }
}

fn run_single(
    lcu: &LcuClient,
    champion_names: &HashMap<i32, String>,
    cli: &Cli,
) -> Result<(), String> {
    println!("{}", "Querying champion select session...".dimmed());

    let session_opt = lcu.get_champ_select_session()?;
    let session = match session_opt {
        Some(s) => s,
        None => {
            return Err(
                "You are not currently in a Champion Select session (or game has not started yet)."
                    .to_string(),
            );
        }
    };

    let masteries = lcu.get_mastery_map()?;
    let options = ExtractOptions::from(cli);
    let (champions, current) =
        extract_available_champions(&session, champion_names, &masteries, &options);

    if cli.json {
        let json_output = serde_json::to_string_pretty(&champions)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
        println!("{}", json_output);
    } else {
        println!(
            "{} ARAM Draft Active | Bench: {} | Teammates: {}",
            "✓".green().bold(),
            session.bench_champions.len(),
            session.my_team.len().saturating_sub(1)
        );
        println!();
        print_champions_table(&champions, &current);
    }

    Ok(())
}

fn run_watch(
    mut lcu: LcuClient,
    mut champion_names: HashMap<i32, String>,
    cli: &Cli,
) -> Result<(), String> {
    println!(
        "{} Monitoring Champion Select (interval: {}s)... Press Ctrl+C to exit.",
        "👀".bold(),
        cli.interval
    );

    let mut last_signature: Option<String> = None;
    let mut cached_masteries: Option<HashMap<i32, ChampionMasteryItem>> = None;
    let mut in_session = false;

    let sleep_dur = Duration::from_secs_f32(cli.interval.max(0.5));
    let options = ExtractOptions::from(cli);

    loop {
        match lcu.get_champ_select_session() {
            Ok(Some(session)) => {
                let sig = generate_signature(&session);
                let changed = last_signature.as_ref() != Some(&sig);

                if changed {
                    if cached_masteries.is_none() {
                        cached_masteries = lcu.get_mastery_map().ok();
                    }

                    let masteries = cached_masteries.as_ref();
                    let dummy_map = HashMap::new();
                    let m_ref = masteries.unwrap_or(&dummy_map);

                    let (champions, current) =
                        extract_available_champions(&session, &champion_names, m_ref, &options);

                    // Clear terminal screen
                    print!("\x1B[2J\x1B[1;1H");

                    if cli.json {
                        if let Ok(json_output) = serde_json::to_string_pretty(&champions) {
                            println!("{}", json_output);
                        }
                    } else {
                        println!(
                            "{} {} | Bench: {} | Teammates: {}",
                            "🎲 ARAM Draft Active".bold().cyan(),
                            chrono_now(),
                            session.bench_champions.len().to_string().yellow(),
                            session.my_team.len().saturating_sub(1).to_string().yellow()
                        );
                        println!();
                        print_champions_table(&champions, &current);
                    }

                    last_signature = Some(sig);
                    in_session = true;
                }
            }
            Ok(None) => {
                if in_session || last_signature.is_none() {
                    print!("\x1B[2J\x1B[1;1H");
                    println!(
                        "{} {} - Waiting for ARAM Champion Select draft to begin...",
                        "⏳".bold(),
                        chrono_now()
                    );
                    in_session = false;
                    last_signature = Some("NONE".to_string());
                    cached_masteries = None;
                }
            }
            Err(e) => {
                // If the connection was refused or lost (e.g. League Client closed or restarted)
                print!("\x1B[2J\x1B[1;1H");
                println!(
                    "{} Connection lost ({}), waiting for League of Legends client...",
                    "⏳".bold(),
                    e.dimmed()
                );
                let (new_client, new_names) =
                    wait_for_lcu(cli.port, cli.auth.clone(), cli.interval);
                lcu = new_client;
                champion_names = new_names;
                last_signature = None;
                cached_masteries = None;
                in_session = false;
            }
        }

        thread::sleep(sleep_dur);
    }
}

fn main() {
    let cli = Cli::parse();

    if cli.once {
        println!("{}", "Connecting to League Client (LCU)...".bold());
        let lcu = match LcuClient::new(cli.port, cli.auth.clone()) {
            Ok(client) => {
                println!(
                    "{} Connected to LCU on port {}",
                    "✓".green().bold(),
                    client.creds.port.to_string().cyan()
                );
                client
            }
            Err(err) => {
                eprintln!("{} {}", "Error:".red().bold(), err);
                std::process::exit(1);
            }
        };

        println!("{}", "Loading champion catalog...".dimmed());
        let champion_names = match lcu.get_champion_map() {
            Ok(names) => names,
            Err(e) => {
                eprintln!(
                    "{} Failed to load champion summary: {}",
                    "Warning:".yellow().bold(),
                    e
                );
                HashMap::new()
            }
        };

        if let Err(e) = run_single(&lcu, &champion_names, &cli) {
            eprintln!("{} {}", "Notice:".yellow().bold(), e);
            std::process::exit(0);
        }
    } else {
        // Default mode: Watch / Live mode (user can just double-click the exe)
        let (lcu, champion_names) = wait_for_lcu(cli.port, cli.auth.clone(), cli.interval);
        if let Err(e) = run_watch(lcu, champion_names, &cli) {
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}
