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
        let token = env::var("PRIVATE_REPO_TOKEN")
            .expect("PRIVATE_REPO_TOKEN still not SET!, Please fix it!");

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
        let token = env::var("PRIVATE_REPO_TOKEN")?;
        let url = format!("https://api.github.com/repos/{}/{}/compare/main...{}", org, repository, sha);

        let client = reqwest::Client::new();
        let res = client.get(url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(USER_AGENT, "TaskBot")
            .send()
            .await?;

        let json: serde_json::Value = res.json().await?;
        let status = json["status"].as_str().unwrap_or("");
        Ok(status == "behind" || status == "identical")
    }
}
fn parse_claim_command(comment: &str) -> Option<ClaimCommand> {
    // valid format: /claim <Task_ID> repository:<repo_name> commit:<commit_HASH>
    let parts: Vec<&str> = comment.split_whitespace().collect();
    if parts.len() == 3 && parts[0] == "/claim" && parts[2].starts_with("commit:") {
        let quest_id = parts[1].to_string();
        let commit_sha = parts[2].replace("commit:", "");
        Some(ClaimCommand { quest_id, commit_sha })
    } else {
        None
    }
}
