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
    
    let (task_id, repository, commit_sha) = parse_command(&args.comment).expect("Invalid format!");

    if !TaskRegistry::validate_commit(&commit_sha, &repository).await? {
        return Err(format!("This commit was not merged into the main branch of the '{}' repository!", repository).into());
    }

    update_player_data(&args.user, &task_id).await?;

    Ok(())
}
