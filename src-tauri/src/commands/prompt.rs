use tauri::State;

use crate::models;
use crate::store;
use crate::AppState;

/// List all prompts.
#[tauri::command]
pub fn list_prompts(
    state: State<'_, AppState>,
) -> Result<Vec<models::Prompt>, String> {
    log::debug!("RS::CMD::prompt | list");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::list_prompts(&db).map_err(|e| e.to_string())
}

/// Create a new prompt.
#[tauri::command]
pub fn create_prompt(
    name: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<models::Prompt, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    let prompt = models::Prompt {
        id: id.clone(),
        name: name.clone(),
        content,
        is_default: false,
        created_at: now,
        updated_at: now,
    };
    log::info!("RS::CMD::prompt | create | id={} name={}", id, name);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::create_prompt(&db, &prompt).map_err(|e| e.to_string())?;
    Ok(prompt)
}

/// Update an existing prompt.
#[tauri::command]
pub fn update_prompt(
    id: String,
    name: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();
    log::info!("RS::CMD::prompt | update | id={}", id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let prompt = models::Prompt {
        id,
        name,
        content,
        is_default: false,
        created_at: 0,
        updated_at: now,
    };
    store::update_prompt(&db, &prompt).map_err(|e| e.to_string())
}

/// Delete a prompt.
#[tauri::command]
pub fn delete_prompt(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("RS::CMD::prompt | delete | id={}", id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::delete_prompt(&db, &id).map_err(|e| e.to_string())
}

/// Set a prompt as default.
#[tauri::command]
pub fn set_default_prompt(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("RS::CMD::prompt | set_default | id={}", id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::set_default_prompt(&db, &id).map_err(|e| e.to_string())
}
