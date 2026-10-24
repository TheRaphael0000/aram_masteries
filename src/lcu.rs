use reqwest::blocking::Client;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use sysinfo::{ProcessesToUpdate, System};

use crate::models::{ChallengeInfo, ChampSelectSession, ChampionMasteryItem, ChampionSummaryItem};

#[derive(Debug, Clone)]
pub struct LcuCredentials {
    pub port: u16,
    pub token: String,
    pub protocol: String,
}

pub struct LcuClient {
    client: Client,
    pub creds: LcuCredentials,
    base_url: String,
}

impl LcuClient {
    pub fn new(custom_port: Option<u16>, custom_token: Option<String>) -> Result<Self, String> {
        let creds = match (custom_port, custom_token) {
            (Some(port), Some(token)) => LcuCredentials {
                port,
                token,
                protocol: "https".to_string(),
            },
            (port_opt, token_opt) => {
                let mut detected = Self::detect_credentials()
                    .ok_or_else(|| "Could not locate League of Legends client. Please make sure the League Client is open and logged in.".to_string())?;
                if let Some(p) = port_opt {
                    detected.port = p;
                }
                if let Some(t) = token_opt {
                    detected.token = t;
                }
                detected
            }
        };

        let base_url = format!("{}://127.0.0.1:{}", creds.protocol, creds.port);

        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            client,
            creds,
            base_url,
        })
    }

    pub fn detect_credentials() -> Option<LcuCredentials> {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let mut candidate_paths: Vec<PathBuf> = Vec::new();

        for process in sys.processes().values() {
            let name = process.name().to_string_lossy();
            if name == "LeagueClientUx.exe" || name == "LeagueClient.exe" {
                if let Some(dir) = process.exe().and_then(|p| p.parent()) {
                    let lockfile = dir.join("lockfile");
                    if lockfile.exists() && !candidate_paths.contains(&lockfile) {
                        candidate_paths.push(lockfile);
                    }
                }
            }
        }

        let default_paths = [
            r"C:\Riot Games\League of Legends\lockfile",
            r"D:\Riot Games\League of Legends\lockfile",
            r"E:\Riot Games\League of Legends\lockfile",
            r"C:\Program Files\Riot Games\League of Legends\lockfile",
            r"C:\Program Files (x86)\Riot Games\League of Legends\lockfile",
        ];

        for p in &default_paths {
            let path = PathBuf::from(p);
            if path.exists() && !candidate_paths.contains(&path) {
                candidate_paths.push(path);
            }
        }

        for path in candidate_paths {
            if let Ok(content) = fs::read_to_string(&path) {
                let parts: Vec<&str> = content.split(':').collect();
                if let Some(port) = parts.get(2).and_then(|p| p.parse::<u16>().ok()) {
                    if let (Some(token), Some(protocol)) = (parts.get(3), parts.get(4)) {
                        return Some(LcuCredentials {
                            port,
                            token: token.to_string(),
                            protocol: protocol.trim().to_string(),
                        });
                    }
                }
            }
        }

        None
    }

    pub fn get_champion_map(&self) -> Result<HashMap<i32, String>, String> {
        let url = format!(
            "{}/lol-game-data/assets/v1/champion-summary.json",
            self.base_url
        );
        let resp = self
            .client
            .get(&url)
            .basic_auth("riot", Some(&self.creds.token))
            .send()
            .map_err(|e| format!("Failed to fetch champions: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Failed to fetch champions: HTTP {}", resp.status()));
        }

        let items: Vec<ChampionSummaryItem> = resp
            .json()
            .map_err(|e| format!("Failed to parse champions JSON: {}", e))?;

        let mut map = HashMap::new();
        for item in items {
            if item.id > 0 {
                map.insert(item.id, item.name);
            }
        }
        Ok(map)
    }

    pub fn get_mastery_map(&self) -> Result<HashMap<i32, ChampionMasteryItem>, String> {
        let url = format!(
            "{}/lol-champion-mastery/v1/local-player/champion-mastery",
            self.base_url
        );
        let resp = self
            .client
            .get(&url)
            .basic_auth("riot", Some(&self.creds.token))
            .send()
            .map_err(|e| format!("Failed to fetch mastery: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Failed to fetch mastery: HTTP {}", resp.status()));
        }

        let items: Vec<ChampionMasteryItem> = resp
            .json()
            .map_err(|e| format!("Failed to parse mastery JSON: {}", e))?;

        let mut map = HashMap::new();
        for item in items {
            map.insert(item.champion_id, item);
        }
        Ok(map)
    }

    pub fn get_champ_select_session(&self) -> Result<Option<ChampSelectSession>, String> {
        let url = format!("{}/lol-champ-select/v1/session", self.base_url);
        let resp = self
            .client
            .get(&url)
            .basic_auth("riot", Some(&self.creds.token))
            .send()
            .map_err(|e| format!("Failed to query champ select session: {}", e))?;

        if resp.status().as_u16() == 404 {
            return Ok(None);
        }

        if !resp.status().is_success() {
            return Err(format!(
                "Champ select query returned HTTP {}",
                resp.status()
            ));
        }

        let session: ChampSelectSession = resp
            .json()
            .map_err(|e| format!("Failed to parse champ select session JSON: {}", e))?;

        Ok(Some(session))
    }

    pub fn get_challenges(&self) -> Result<HashMap<String, ChallengeInfo>, String> {
        let url = format!(
            "{}/lol-challenges/v1/challenges/local-player",
            self.base_url
        );
        let resp = self
            .client
            .get(&url)
            .basic_auth("riot", Some(&self.creds.token))
            .send()
            .map_err(|e| format!("Failed to fetch challenges: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!(
                "Failed to fetch challenges: HTTP {}",
                resp.status()
            ));
        }

        resp.json()
            .map_err(|e| format!("Failed to parse challenges JSON: {}", e))
    }

    pub fn get_arac_challenge(&self) -> HashSet<i32> {
        // If fetching challenges fails, return an empty set
        let challenges = match self.get_challenges() {
            Ok(c) => c,
            Err(_) => return HashSet::new(),
        };

        // Try lookup by "101301", collect if found, otherwise return empty set
        if let Some(c) = challenges.get("101301") {
            c.completed_ids.iter().copied().collect()
        } else {
            HashSet::new()
        }
    }

    pub fn get_raw(&self, endpoint: &str) -> Result<String, String> {
        let url = format!("{}{}", self.base_url, endpoint);
        let resp = self
            .client
            .get(&url)
            .basic_auth("riot", Some(&self.creds.token))
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Status: {}", resp.status()));
        }

        resp.text()
            .map_err(|e| format!("Failed to read text: {}", e))
    }
}
