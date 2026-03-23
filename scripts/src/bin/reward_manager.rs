use clap::Parser;
use reward_manager::{FullDatabase, Player, parse_claim_command};

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
    let args = Args::parse();
    let db_path = "../database.json";
   
    let mut db = FullDatabase::load(db_path)
        .expect("Could not load '../database.json'. Please run gen_db first!");

    if let Some(command) = parse_claim_command(&args.comment) {
        // 1. Verify Task exists
        if let Some(task) = db.tasks.get(&command.task_id).cloned() {
            println!("🔍 Validating task {} for user {}...", task.id, args.user);

            // 2. Validate commit in the private repo
            // Note: Org is hardcoded as 'RDAsaBranca' for now based on previous code
            let is_valid = db.validate_commit("RDAsaBranca", &command.commit_sha, &command.repository).await?;

            if is_valid {
                println!("✅ Task {} validated!", task.id);

                // 3. Update or create player
                let player = db.players.entry(args.user.clone())
                    .or_insert_with(|| Player::new(&args.user));

                // Check if task already completed (if not repeatable)
                let class_key = task.class.to_lowercase().replace(" ", "_");
                if let Some(class) = player.classes.get(&class_key) {
                    if !task.repeatable && class.completed_tasks.contains(&task.id) {
                        println!("⚠️ User {} already completed task {}.", args.user, task.id);
                        return Ok(());
                    }
                }

                player.update_xp(&class_key, &task.id, task.xp);
                println!("🎉 User {} rewarded with {} XP in {}!", args.user, task.xp, class_key);

                // 4. Save results
                db.save(db_path)?;
                println!("💾 Database updated successfully.");
            } else {
                println!("❌ Validation failed for commit {}.", command.commit_sha);
            }
        } else {
            println!("❌ Task ID '{}' not found in registry.", command.task_id);
        }
    } else {
        println!("ℹ️ Comment '{}' is not a valid claim command.", args.comment);
    }

    Ok(())
}
