use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Quest {
    id: String,
    title: String,
    class: String,
    xp: u32,
}

struct ClaimCommand {
    quest_id: String,
    repository: String,
    commit_sha: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestRegistry {
    pub quests: HashMap<String, Quest>,
}

impl QuestRegistry {
    pub async fn validate_private_work(
        &self,
        org: &str,
        repo: &str,
        commit_sha: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // Recupera o Token de Acesso Pessoal (PAT) das variáveis de ambiente
        let token = env::var("PRIVATE_REPO_TOKEN")
            .expect("PRIVATE_REPO_TOKEN still not SET!, Please fix it!");

        let url = format!("https://api.github.com/repos/{}/{}/commits/{}", org, repo, commit_sha);
        let client = reqwest::Client::new();

        let response = client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(USER_AGENT, "QuestBot-Rust-Validator")
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
}
// Exemplo de lógica para processar o comando vindo do comentário
fn parse_claim_command(comment: &str) -> Option<ClaimCommand> {
    // Espera o formato: /claim ID commit:HASH
    let parts: Vec<&str> = comment.split_whitespace().collect();
    if parts.len() == 3 && parts[0] == "/claim" && parts[2].starts_with("commit:") {
        let quest_id = parts[1].to_string();
        let commit_sha = parts[2].replace("commit:", "");
        Some(ClaimCommand { quest_id, commit_sha })
    } else {
        None
    }
}
