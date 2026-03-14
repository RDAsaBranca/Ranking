use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: String,
    title: String,
    class: String,
    repeatable: bool,
    unique: bool,
    xp: u32,
}

impl TaskRegistry {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(path)?;
        let registry: TaskRegistry = serde_json::from_str(&data)?;
        Ok(registry)
    }
}

struct ClaimCommand {
    task_id: String,
    repository: String,
    commit_sha: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskRegistry {
    pub tasks: HashMap<String, Task>,
}

impl TaskRegistry {
    pub async fn validate_private_work(
        &self,
        org: &str,
        repo: &str,
        commit_sha: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let token = env::var(format!("{}_REPO_TOKEN", repo.to_uppercase()))
            .expect(format!("{}_REPO_TOKEN still not SET!, Please fix it!", repo.to_uppercase()));

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
        )-> Result<bool, Box<dyn std::error::Error>> {
        let env_var = format!("{}_REPO_TOKEN", repository.to_uppercase());
        let token = env::var(&env_var)
            .map_err(|_| format!("Variable {} not found!, env_var"))?;

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
fn parse_claim_command(comment: &str) -> Option<ClaimCommand> {
    // valid format: /claim <Task_ID> repository:<repo_name> commit:<commit_HASH>
    let parts: Vec<&str> = comment.split_whitespace().collect();
    if parts.len() == 4 && parts[0] == "/claim" 
        && parts[2].starts_with("repository:") 
        && parts[3].starts_with("commit:") {
        
        let quest_id = parts[1].to_string();
        let repo = parts[2].replace("repository:", "");
        let commit_sha = parts[3].replace("commit:", "");
        Some(ClaimCommand { quest_id, repo, commit_sha })
    } else {
        None
    }
}
