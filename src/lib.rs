pub mod lcu;
pub mod models;

use std::collections::HashMap;
use colored::*;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, CellAlignment, Color, ContentArrangement, Row, Table};

use crate::models::{AvailableChampion, ChampSelectSession, ChampionMasteryItem, ChampionSource};

#[derive(Debug, Clone, Default)]
pub struct ExtractOptions {
    pub bench_only: bool,
    pub team_only: bool,
    pub no_current: bool,
}

pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*ch);
    }
    result
}

pub fn extract_available_champions(
    session: &ChampSelectSession,
    champion_names: &HashMap<i32, String>,
    masteries: &HashMap<i32, ChampionMasteryItem>,
    options: &ExtractOptions,
) -> (Vec<AvailableChampion>, Option<AvailableChampion>) {
    let mut available = Vec::new();
    let mut current_champion: Option<AvailableChampion> = None;

    // 1. Current player champion
    for member in &session.my_team {
        if member.cell_id == session.local_player_cell_id && member.champion_id > 0 {
            let name = champion_names
                .get(&member.champion_id)
                .cloned()
                .unwrap_or_else(|| format!("Champion #{}", member.champion_id));
            let mastery = masteries.get(&member.champion_id);
            let item = AvailableChampion {
                champion_id: member.champion_id,
                name,
                source: ChampionSource::Current,
                mastery_level: mastery.map(|m| m.champion_level).unwrap_or(0),
                mastery_points: mastery.map(|m| m.champion_points).unwrap_or(0),
                points_until_next_level: mastery.map(|m| m.champion_points_until_next_level).unwrap_or(0),
                highest_grade: mastery.and_then(|m| m.highest_grade.clone()),
            };
            current_champion = Some(item.clone());
            if !options.no_current && !options.bench_only && !options.team_only {
                available.push(item);
            }
        }
    }

    // 2. Bench Champions (Pool)
    if !options.team_only {
        for bench in &session.bench_champions {
            if bench.champion_id > 0 {
                let name = champion_names
                    .get(&bench.champion_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Champion #{}", bench.champion_id));
                let mastery = masteries.get(&bench.champion_id);
                available.push(AvailableChampion {
                    champion_id: bench.champion_id,
                    name,
                    source: ChampionSource::Bench,
                    mastery_level: mastery.map(|m| m.champion_level).unwrap_or(0),
                    mastery_points: mastery.map(|m| m.champion_points).unwrap_or(0),
                    points_until_next_level: mastery.map(|m| m.champion_points_until_next_level).unwrap_or(0),
                    highest_grade: mastery.and_then(|m| m.highest_grade.clone()),
                });
            }
        }
    }

    // 3. Teammates Champions
    if !options.bench_only {
        for member in &session.my_team {
            if member.cell_id != session.local_player_cell_id && member.champion_id > 0 {
                let name = champion_names
                    .get(&member.champion_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Champion #{}", member.champion_id));
                let mastery = masteries.get(&member.champion_id);
                available.push(AvailableChampion {
                    champion_id: member.champion_id,
                    name,
                    source: ChampionSource::Teammate,
                    mastery_level: mastery.map(|m| m.champion_level).unwrap_or(0),
                    mastery_points: mastery.map(|m| m.champion_points).unwrap_or(0),
                    points_until_next_level: mastery.map(|m| m.champion_points_until_next_level).unwrap_or(0),
                    highest_grade: mastery.and_then(|m| m.highest_grade.clone()),
                });
            }
        }
    }

    // Sort by mastery score descending, then mastery level descending, then name ascending
    available.sort_by(|a, b| {
        b.mastery_points
            .cmp(&a.mastery_points)
            .then_with(|| b.mastery_level.cmp(&a.mastery_level))
            .then_with(|| a.name.cmp(&b.name))
    });

    (available, current_champion)
}

pub fn print_champions_table(champions: &[AvailableChampion], current: &Option<AvailableChampion>) {
    if champions.is_empty() {
        println!("{}", "No available champions found matching criteria.".yellow());
        return;
    }

    if let Some(curr) = current {
        println!(
            "{} {} (Level {}, {} pts)",
            "Currently Assigned:".bold().cyan(),
            curr.name.bold().green(),
            curr.mastery_level,
            format_number(curr.mastery_points).yellow()
        );
        println!();
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("#").set_alignment(CellAlignment::Center),
            Cell::new("Champion").set_alignment(CellAlignment::Left),
            Cell::new("Source").set_alignment(CellAlignment::Center),
            Cell::new("Mastery Level").set_alignment(CellAlignment::Right),
            Cell::new("Mastery Points").set_alignment(CellAlignment::Right),
            Cell::new("Next Level in").set_alignment(CellAlignment::Right),
            Cell::new("Season Grade").set_alignment(CellAlignment::Center),
        ]);

    for (index, champ) in champions.iter().enumerate() {
        let (source_str, source_color) = match champ.source {
            ChampionSource::Current => ("[CURRENT]", Color::Green),
            ChampionSource::Bench => ("[BENCH POOL]", Color::Cyan),
            ChampionSource::Teammate => ("[TEAMMATE]", Color::Yellow),
        };

        let next_pts = if champ.points_until_next_level > 0 {
            format!("+{}", format_number(champ.points_until_next_level as u64))
        } else {
            "-".to_string()
        };

        let grade = champ.highest_grade.as_deref().unwrap_or("-");

        let mut row = Row::new();
        row.add_cell(Cell::new(format!("{}", index + 1)).set_alignment(CellAlignment::Center));
        row.add_cell(
            Cell::new(&champ.name)
                .set_alignment(CellAlignment::Left)
                .fg(if champ.source == ChampionSource::Current {
                    Color::Green
                } else if champ.mastery_points > 100_000 {
                    Color::Yellow
                } else {
                    Color::White
                }),
        );
        row.add_cell(Cell::new(source_str).set_alignment(CellAlignment::Center).fg(source_color));
        row.add_cell(Cell::new(format!("{}", champ.mastery_level)).set_alignment(CellAlignment::Right));
        row.add_cell(
            Cell::new(format_number(champ.mastery_points))
                .set_alignment(CellAlignment::Right)
                .fg(if champ.mastery_points > 0 { Color::Yellow } else { Color::DarkGrey }),
        );
        row.add_cell(Cell::new(next_pts).set_alignment(CellAlignment::Right));
        row.add_cell(Cell::new(grade).set_alignment(CellAlignment::Center));

        table.add_row(row);
    }

    println!("{table}");
}

pub fn generate_signature(session: &ChampSelectSession) -> String {
    let mut bench_ids: Vec<i32> = session.bench_champions.iter().map(|b| b.champion_id).collect();
    bench_ids.sort();
    let team_picks: Vec<String> = session
        .my_team
        .iter()
        .map(|m| format!("{}:{}", m.cell_id, m.champion_id))
        .collect();
    format!("{:?}_{:?}", bench_ids, team_picks)
}
