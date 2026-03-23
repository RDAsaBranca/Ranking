use reward_manager::{FullDatabase, Task};
use std::fs;
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = "../database.json";
    let mut db = FullDatabase::load(db_path).unwrap_or_else(|_| {
        FullDatabase {
            tasks: std::collections::HashMap::new(),
            players: std::collections::HashMap::new(),
        }
    });

    for entry in WalkDir::new("../content")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md")) 
    {
        let content = fs::read_to_string(entry.path())?;
        if let Some(fm_block) = content.split("---").nth(1) {
            let mut task: Task = serde_yaml::from_str(fm_block)?;
            // Normalize class name for mapping (e.g. "Firmware" -> "firmware_artificier")
            // This is a POC logic; we can refine this mapping later.
            if task.class == "Firmware" {
                task.class = "firmware_artificier".to_string();
            }
            
            db.tasks.insert(task.id.clone(), task);
        }
    }

    db.save(db_path)?;
    println!("✅ Registry updated with {} tasks.", db.tasks.len());
    Ok(())
}
