use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
struct Registry {
    tasks: HashMap<String, Task>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = Registry { tasks: HashMap::new() };

    for entry in WalkDir::new("../content")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md")) 
    {
        let content = fs::read_to_string(entry.path())?;
        if let Some(fm_block) = content.split("---").nth(1) {
            let task: Task = serde_yaml::from_str(fm_block)?;
            registry.tasks.insert(task.id.clone(), task);
        }
    }

    fs::write("../database.json", serde_json::to_string_pretty(&registry)?)?;
    Ok(())
}
