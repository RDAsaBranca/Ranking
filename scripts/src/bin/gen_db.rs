use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
struct Registry {
    quests: HashMap<String, Quest>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = Registry { quests: HashMap::new() };

    for entry in WalkDir::new("../content")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md")) 
    {
        let content = fs::read_to_string(entry.path())?;
        if let Some(fm_block) = content.split("---").nth(1) {
            let quest: Quest = serde_yaml::from_str(fm_block)?;
            registry.quests.insert(quest.id.clone(), quest);
        }
    }

    fs::write("../database.json", serde_json::to_string_pretty(&registry)?)?;
    Ok(())
}
