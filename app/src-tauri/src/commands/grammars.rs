use std::collections::HashSet;

use dme_core::preview::{
    proposed_grammars::PROPOSED_GRAMMAR_SOURCES, tree_sitter_grammars::TreeSitterGrammarsManager,
};
use serde::Serialize;

#[derive(Serialize)]
pub enum InstalledStatus {
    NotInstalled,
    // We dont manage the "Installing" status in Rust, only on the frontend
    Installed,
}

#[derive(Serialize)]
pub struct GrammarState {
    id: String,
    link: String,
    status: InstalledStatus,
}

#[tauri::command]
pub fn get_grammars_list() -> Result<Vec<GrammarState>, String> {
    let mut manager = TreeSitterGrammarsManager::new()?;
    let list = manager.list_installed_langs()?;
    let installed_map: HashSet<String> = HashSet::from_iter(list);
    let states = PROPOSED_GRAMMAR_SOURCES.clone();
    let mut result = states
        .iter()
        .map(|(key, value)| GrammarState {
            id: key.to_string(),
            link: value.to_string(),
            status: {
                if installed_map.contains(*key) {
                    InstalledStatus::Installed
                } else {
                    InstalledStatus::NotInstalled
                }
            },
        })
        .collect::<Vec<GrammarState>>();
    result.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(result)
}

#[tauri::command(async)]
pub fn install_grammar(id: &str) -> Result<(), String> {
    let mut manager = TreeSitterGrammarsManager::new()?;
    let link = PROPOSED_GRAMMAR_SOURCES
        .get(id)
        .ok_or(format!("No proposed grammar with id {id}"))?;
    manager.install(link)?;
    Ok(())
}

#[tauri::command(async)]
pub fn remove_grammar(id: &str) -> Result<(), String> {
    let mut manager = TreeSitterGrammarsManager::new()?;
    let link = PROPOSED_GRAMMAR_SOURCES
        .get(id)
        .ok_or(format!("No proposed grammar with id {id}"))?;
    manager.delete(id)?;
    Ok(())
}

#[tauri::command(async)]
pub fn grammars_folder() -> Result<String, String> {
    let manager = TreeSitterGrammarsManager::new()?;
    Ok(manager
        .get_grammars_folder()
        .to_str()
        .unwrap_or("??")
        .to_string())
}
