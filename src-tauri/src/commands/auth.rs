use crate::domain::{audit, auth};
use crate::domain::auth::Operator;
use crate::error::Result;
use crate::state::AppState;
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct SetupInput {
    pub username: String,
    pub password: String,
    pub full_name: String,
}

#[tauri::command]
pub fn needs_setup(state: State<'_, AppState>) -> Result<bool> {
    Ok(!auth::has_any_operator(&state.db())?)
}

#[tauri::command]
pub fn setup_first_manager(state: State<'_, AppState>, input: SetupInput) -> Result<Operator> {
    let op = auth::create_first_manager(
        &mut state.db(),
        &input.username,
        &input.password,
        &input.full_name,
    )?;
    state.set_user(Some(op.clone()));
    Ok(op)
}

#[tauri::command]
pub fn login(state: State<'_, AppState>, username: String, password: String) -> Result<Operator> {
    let op = auth::verify_login(&state.db(), &username, &password)?;
    state.set_user(Some(op.clone()));
    Ok(op)
}

#[tauri::command]
pub fn logout(state: State<'_, AppState>) -> Result<()> {
    if let Some(op) = state.current_user() {
        audit::log(&state.db(), None, Some(&op.id), "logout", None)?;
    }
    state.set_user(None);
    Ok(())
}

#[tauri::command]
pub fn current_user(state: State<'_, AppState>) -> Option<Operator> {
    state.current_user()
}
