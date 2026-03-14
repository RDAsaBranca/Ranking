use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs};
use reqwest::header::{AUTHORIZATION, USER_AGENT};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    user: String,
    #[arg(short, long)]
    sha: String,
    #[arg(short, long)]
    comment: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(); // "/claim FW-01 repository:repo commit:hash"
   
    let registry = TaskRegistry::load("database.json").
        .expect("Could not load properly the 'database.json' file, please run gen_db first!");

    if let Some(command) = parse_claim_command(&args.comment) {
        if let Some(task) = registry.task.get(&command.task_id) {
            if registry.validate_commit("RDAsaBranca", &command.commit_sha, &command.repository).await? {
                println!("Task {} validated for user {}", task.id, args.user);
            }
        }
    }

    update_player_data(&args.user, &task_id).await?;

    Ok(())
}
