mod commands;
mod domain;
mod db;
mod error;
mod hardware;
mod state;
mod util;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            app.manage(AppState::init(dir)?);
            commands::reader::start_on_boot(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::needs_setup,
            commands::auth::setup_first_manager,
            commands::auth::login,
            commands::auth::logout,
            commands::auth::current_user,
            commands::cards::list_cards,
            commands::cards::create_card,
            commands::entry::prepare_entry,
            commands::entry::recapture,
            commands::entry::cancel_pending,
            commands::entry::confirm_entry,
            commands::exit::prepare_exit,
            commands::exit::confirm_exit,
            commands::exit::refuse_exit,
            commands::reader::list_serial_ports,
            commands::reader::get_reader_config,
            commands::reader::set_reader_config,
        ])
        .run(tauri::generate_context!())
        .expect("không khởi động được ứng dụng");
}
