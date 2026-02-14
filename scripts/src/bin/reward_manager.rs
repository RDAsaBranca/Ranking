use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs};
use reqwest::header::{AUTHORIZATION, USER_AGENT};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    user: String,
    #[arg(short, long)]
    comment: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(); // "/claim FW-01 repostory:repo commit:hash"
    
    let (quest_id, commit_sha) = parse_command(&args.comment).expect("Formato inválido");

    if !validate_commit(&commit_sha).await? {
        return Err("Commit não mergeado na main do repo privado!".into());
    }

    update_player_data(&args.user, &quest_id).await?;

    Ok(())
}

async fn validate_commit(sha: &str, repository: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let token = env::var("PRIVATE_REPO_TOKEN")?;
    let url = format!("https://api.github.com/repos/RDAsaBranca/{}/compare/main...{}", repository, sha);

    let client = reqwest::Client::new();
    let res = client.get(url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header(USER_AGENT, "RustBot")
        .send().await?;

    let json: serde_json::Value = res.json().await?;
    let status = json["status"].as_str().unwrap_or("");
    Ok(status == "behind" || status == "identical")
}
