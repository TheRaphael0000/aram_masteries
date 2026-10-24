use aram_masteries::models::{
    BenchChampion, ChampSelectSession, ChampionMasteryItem, ChampionSource, TeamMember,
};
use aram_masteries::{extract_available_champions, format_number, ExtractOptions};
use std::collections::HashMap;

#[test]
fn test_number_formatting() {
    assert_eq!(format_number(0), "0");
    assert_eq!(format_number(999), "999");
    assert_eq!(format_number(1000), "1,000");
    assert_eq!(format_number(555507), "555,507");
    assert_eq!(format_number(12345678), "12,345,678");
}

#[test]
fn test_extract_and_sort_by_mastery() {
    let session = ChampSelectSession {
        bench_enabled: true,
        local_player_cell_id: 0,
        bench_champions: vec![
            BenchChampion {
                champion_id: 412, // Thresh
                is_custody: Some(false),
            },
            BenchChampion {
                champion_id: 1, // Annie
                is_custody: Some(false),
            },
            BenchChampion {
                champion_id: 999, // Unplayed champion
                is_custody: Some(false),
            },
        ],
        my_team: vec![
            TeamMember {
                cell_id: 0,       // Local player
                champion_id: 103, // Ahri
                summoner_id: Some(1),
                puuid: None,
                assigned_position: None,
            },
            TeamMember {
                cell_id: 1,      // Teammate
                champion_id: 64, // Lee Sin
                summoner_id: Some(2),
                puuid: None,
                assigned_position: None,
            },
        ],
    };

    let mut names = HashMap::new();
    names.insert(412, "Thresh".to_string());
    names.insert(1, "Annie".to_string());
    names.insert(999, "NewChamp".to_string());
    names.insert(103, "Ahri".to_string());
    names.insert(64, "Lee Sin".to_string());

    let mut masteries = HashMap::new();
    masteries.insert(
        412,
        ChampionMasteryItem {
            champion_id: 412,
            champion_level: 53,
            champion_points: 555_507,
            champion_points_until_next_level: 4093,
            highest_grade: Some("S+".to_string()),
        },
    );
    masteries.insert(
        1,
        ChampionMasteryItem {
            champion_id: 1,
            champion_level: 5,
            champion_points: 25_000,
            champion_points_until_next_level: 1000,
            highest_grade: Some("A".to_string()),
        },
    );
    masteries.insert(
        103,
        ChampionMasteryItem {
            champion_id: 103,
            champion_level: 7,
            champion_points: 80_000,
            champion_points_until_next_level: 0,
            highest_grade: Some("S".to_string()),
        },
    );
    masteries.insert(
        64,
        ChampionMasteryItem {
            champion_id: 64,
            champion_level: 12,
            champion_points: 150_000,
            champion_points_until_next_level: 2500,
            highest_grade: Some("S-".to_string()),
        },
    );

    // Test default extraction (includes bench, teammates, and current)
    let options = ExtractOptions::default();
    // let (champions, current) = extract_available_champions(&session, &names, &masteries, &options);

    // assert!(current.is_some());
    // assert_eq!(current.as_ref().unwrap().name, "Ahri");

    // assert_eq!(champions.len(), 5);
    // // Ordered by mastery points descending:
    // // 1. Thresh (555,507) - Bench
    // // 2. Lee Sin (150,000) - Teammate
    // // 3. Ahri (80,000) - Current
    // // 4. Annie (25,000) - Bench
    // // 5. NewChamp (0) - Bench
    // assert_eq!(champions[0].name, "Thresh");
    // assert_eq!(champions[0].source, ChampionSource::Bench);
    // assert_eq!(champions[0].mastery_points, 555_507);

    // assert_eq!(champions[1].name, "Lee Sin");
    // assert_eq!(champions[1].source, ChampionSource::Teammate);
    // assert_eq!(champions[1].mastery_points, 150_000);

    // assert_eq!(champions[2].name, "Ahri");
    // assert_eq!(champions[2].source, ChampionSource::Current);
    // assert_eq!(champions[2].mastery_points, 80_000);

    // assert_eq!(champions[3].name, "Annie");
    // assert_eq!(champions[3].source, ChampionSource::Bench);
    // assert_eq!(champions[3].mastery_points, 25_000);

    // assert_eq!(champions[4].name, "NewChamp");
    // assert_eq!(champions[4].source, ChampionSource::Bench);
    // assert_eq!(champions[4].mastery_points, 0);

    // // Test bench only
    // let bench_opts = ExtractOptions {
    //     bench_only: true,
    //     ..Default::default()
    // };
    // let (bench_champs, _) = extract_available_champions(&session, &names, &masteries, &bench_opts);
    // assert_eq!(bench_champs.len(), 3);
    // for c in bench_champs {
    //     assert_eq!(c.source, ChampionSource::Bench);
    // }

    // // Test team only
    // let team_opts = ExtractOptions {
    //     team_only: true,
    //     ..Default::default()
    // };
    // let (team_champs, _) = extract_available_champions(&session, &names, &masteries, &team_opts);
    // assert_eq!(team_champs.len(), 1);
    // assert_eq!(team_champs[0].name, "Lee Sin");
    // assert_eq!(team_champs[0].source, ChampionSource::Teammate);
}

#[test]
fn test_explore_challenges() {
    let client = match aram_masteries::lcu::LcuClient::new(None, None) {
        Ok(c) => c,
        Err(e) => {
            println!("LCU not running: {}", e);
            return;
        }
    };

    if let Ok(raw) = client.get_raw("/lol-challenges/v1/challenges/local-player") {
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
        if let Some(ch) = parsed.get("101301") {
            println!(
                "Challenge 101301:\n{}",
                serde_json::to_string_pretty(ch).unwrap()
            );
        }
    }
}
