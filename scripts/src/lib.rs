use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub class: String,
    pub repeatable: bool,
    pub unique: bool,
    pub xp: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerClass {
    pub level: u32,
    pub exp: u32,
    pub name: String,
    pub completed_tasks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub name: String,
    pub classes: HashMap<String, PlayerClass>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FullDatabase {
    pub tasks: HashMap<String, Task>,
    pub players: HashMap<String, Player>,
}

impl Player {
    pub fn new(name: &str) -> Self {
        let mut classes = HashMap::new();
        let class_names = vec![
            "firmware_artificier", "equilibrium_warden", "spatial_seer",
            "signal_voyager", "foundry_smith", "sage_researcher",
            "energy_channeler", "trace_alchemist"
        ];

        for class in class_names {
            classes.insert(class.to_string(), PlayerClass {
                level: 0,
                exp: 0,
                name: "Novice".to_string(),
                completed_tasks: Vec::new(),
            });
        }

        Player {
            name: name.to_string(),
            classes,
        }
    }

    pub fn update_xp(&mut self, class_name: &str, task_id: &str, xp: u32) {
        if let Some(class) = self.classes.get_mut(class_name) {
            class.exp += xp;
            class.completed_tasks.push(task_id.to_string());

            // POC Level Logic: 0, 100, 300, 600, 1000, 1500
            class.level = match class.exp {
                e if e >= 1500 => 5,
                e if e >= 1000 => 4,
                e if e >= 600 => 3,
                e if e >= 300 => 2,
                e if e >= 100 => 1,
                _ => 0,
            };

            class.name = match class.level {
                5 => "Master".to_string(),
                4 => "Expert".to_string(),
                3 => "Adept".to_string(),
                2 => "Apprentice".to_string(),
                1 => "Initiate".to_string(),
                _ => "Novice".to_string(),
            };
        }
    }
}

impl FullDatabase {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if !std::path::Path::new(path).exists() {
            return Err(format!("Database file not found: {}", path).into());
        }
        let data = std::fs::read_to_string(path)?;
        let db: FullDatabase = serde_json::from_str(&data)?;
        Ok(db)
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub async fn validate_private_work(
        &self,
        org: &str,
        repo: &str,
        commit_sha: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let env_var = format!("{}_REPO_TOKEN", repo.to_uppercase());
        let token = env::var(&env_var)
            .expect(&format!("Variable {} not found!", env_var));

        let url = format!("https://api.github.com/repos/{}/{}/commits/{}", org, repo, commit_sha);
        let client = reqwest::Client::new();

        let response = client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(USER_AGENT, "TaskBot")
            .send()
            .await?;

        if response.status().is_success() {
            println!("✅ Prova de trabalho validada: Commit {} encontrado.", commit_sha);
            Ok(true)
        } else {
            println!("❌ Falha na validação: Commit {} não encontrado ou sem acesso.", commit_sha);
            Ok(false)
        }
    }

    pub async fn validate_commit(
        &self,
        org: &str,
        sha: &str, 
        repository: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let env_var = format!("{}_REPO_TOKEN", repository.to_uppercase());
        let token = env::var(&env_var)
            .map_err(|_| format!("Variable {} not found!", env_var))?;

        let url = format!("https://api.github.com/repos/{}/{}/compare/main...{}", org, repository, sha);

        let client = reqwest::Client::new();
        let res = client.get(url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(USER_AGENT, "TaskBot")
            .send()
            .await?;
        
        if !res.status().is_success() {
            println!("❌ Erro na API do GitHub: Status {}", res.status());
            return Ok(false);
        }
        let json: serde_json::Value = res.json().await?;
        let status = json["status"].as_str().unwrap_or("");
        Ok(status == "behind" || status == "identical")
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimCommand {
    pub task_id: String,
    pub repository: String,
    pub commit_sha: String,
}

pub fn parse_claim_command(comment: &str) -> Option<ClaimCommand> {
    // valid format: /claim <Task_ID> repository:<repo_name> commit:<commit_HASH>
    let parts: Vec<&str> = comment.split_whitespace().collect();
    if parts.len() == 4 && parts[0] == "/claim" 
        && parts[2].starts_with("repository:") 
        && parts[3].starts_with("commit:") {
        
        let task_id = parts[1].to_string();
        let repository = parts[2].replace("repository:", "");
        let commit_sha = parts[3].replace("commit:", "");
        Some(ClaimCommand { task_id, repository, commit_sha })
    } else {
        None
    }
}
