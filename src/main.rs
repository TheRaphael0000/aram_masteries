use clap::Parser;
use colored::*;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use aram_masteries::lcu::LcuClient;
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
                    "Connected to League Client (LCU) on port {}",
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
                    println!("Waiting for League of Legends client to launch and log in...",);
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
    let arac_completed_ids = lcu.get_arac_challenge();

    let options = ExtractOptions::from(cli);
    let (champions, current) = extract_available_champions(
        &session,
        champion_names,
        &masteries,
        &arac_completed_ids,
        &options,
    );

    if cli.json {
        let json_output = serde_json::to_string_pretty(&champions)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
        println!("{}", json_output);
    } else {
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
        "Monitoring Champion Select (interval: {}s)... Press Ctrl+C to exit.",
        cli.interval
    );

    let mut last_signature: Option<String> = None;
    let mut in_session = false;
    let mut was_not_in_champ_select = true;
    let mut arac_completed_ids = lcu.get_arac_challenge();
    let mut masteries = lcu.get_mastery_map()?;

    let sleep_dur = Duration::from_secs_f32(cli.interval.max(0.5));
    let options = ExtractOptions::from(cli);

    loop {
        match lcu.get_champ_select_session() {
            Ok(Some(session)) => {
                // update these if we were not in champ select before
                if was_not_in_champ_select {
                    arac_completed_ids = lcu.get_arac_challenge();
                    masteries = lcu.get_mastery_map()?;
                }
                let sig = generate_signature(&session);
                let changed = last_signature.as_ref() != Some(&sig);

                if changed {
                    let (champions, current) = extract_available_champions(
                        &session,
                        &champion_names,
                        &masteries,
                        &arac_completed_ids,
                        &options,
                    );

                    // Clear terminal screen
                    print!("\x1B[2J\x1B[1;1H");

                    if cli.json {
                        if let Ok(json_output) = serde_json::to_string_pretty(&champions) {
                            println!("{}", json_output);
                        }
                    } else {
                        print_champions_table(&champions, &current);
                    }

                    last_signature = Some(sig);
                    in_session = true;
                    was_not_in_champ_select = false;
                }
            }
            Ok(None) => {
                if in_session || last_signature.is_none() {
                    print!("\x1B[2J\x1B[1;1H");
                    println!("Waiting for ARAM Champion Select draft to begin...",);
                    in_session = false;
                    last_signature = Some("NONE".to_string());
                }
                was_not_in_champ_select = true;
            }
            Err(e) => {
                // If the connection was refused or lost (e.g. League Client closed or restarted)
                print!("\x1B[2J\x1B[1;1H");
                println!(
                    "Connection lost ({}), waiting for League of Legends client...",
                    e.dimmed()
                );
                let (new_client, new_names) =
                    wait_for_lcu(cli.port, cli.auth.clone(), cli.interval);
                lcu = new_client;
                champion_names = new_names;
                last_signature = None;
                in_session = false;
                was_not_in_champ_select = true;
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
                    "Connected to LCU on port {}",
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
