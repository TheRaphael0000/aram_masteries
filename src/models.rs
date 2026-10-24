use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionSummaryItem {
    pub id: i32,
    pub name: String,
    #[serde(default)]
    pub alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionMasteryItem {
    pub champion_id: i32,
    #[serde(default)]
    pub champion_level: u32,
    #[serde(default)]
    pub champion_points: u64,
    #[serde(default)]
    pub champion_points_until_next_level: i64,
    #[serde(default)]
    pub highest_grade: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchChampion {
    pub champion_id: i32,
    #[serde(default)]
    pub is_custody: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamMember {
    pub cell_id: i64,
    pub champion_id: i32,
    #[serde(default)]
    pub summoner_id: Option<u64>,
    #[serde(default)]
    pub puuid: Option<String>,
    #[serde(default)]
    pub assigned_position: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectSession {
    #[serde(default)]
    pub bench_enabled: bool,
    #[serde(default)]
    pub bench_champions: Vec<BenchChampion>,
    #[serde(default)]
    pub my_team: Vec<TeamMember>,
    #[serde(default)]
    pub local_player_cell_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChampionSource {
    Current,
    Bench,
    Teammate,
}

impl std::fmt::Display for ChampionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChampionSource::Current => write!(f, "Current"),
            ChampionSource::Bench => write!(f, "Bench Pool"),
            ChampionSource::Teammate => write!(f, "Teammate"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableChampion {
    pub champion_id: i32,
    pub name: String,
    pub source: ChampionSource,
    pub mastery_level: u32,
    pub mastery_points: u64,
    pub points_until_next_level: i64,
    pub highest_grade: Option<String>,
    #[serde(default, alias = "all_random_all_champions_completed")]
    pub arac_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeInfo {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub completed_ids: Vec<i32>,
    #[serde(default)]
    pub current_level: Option<String>,
    #[serde(default)]
    pub current_value: Option<f64>,
    #[serde(default)]
    pub current_threshold: Option<f64>,
    #[serde(default)]
    pub next_threshold: Option<f64>,
    #[serde(default)]
    pub retire_timestamp: Option<u64>,
}
