mod bot;
mod commands;
mod emit;
mod launch;
mod modules;
mod other;
mod result;

use commands::*;
use emit::emit_event_loop;
use modules::cache::{extract_cache_meta, load_cache, save_cache};
use modules::config::{create_public_config, initialize_configs_dir, load_configs};
use modules::proxy::{check_proxies, collect_proxies};
use modules::storage::{open_directory, read_file, remove_file, save_file};

/// Запуск приложения
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    .setup(|app| {
      let handle = app.handle().clone();

      std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(emit_event_loop(handle));
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      exit,
      is_dev_mode,
      launch_bots,
      stop_bots,
      get_bot_profiles,
      quick_operation,
      get_radar_data,
      save_radar_data,
      follow_radar_target,
      set_group,
      get_active_bots_count,
      get_memory_usage,
      control_bots,
      quick_task,
      execute_script,
      stop_script,
      get_server_info,
      open_url,
      download_json,
      download_text,
      read_text_file,
      set_discord_rpc,
      collect_proxies,
      check_proxies,
      extract_cache_meta,
      save_cache,
      load_cache,
      initialize_configs_dir,
      load_configs,
      create_public_config,
      read_file,
      save_file,
      remove_file,
      open_directory,
    ])
    .run(tauri::generate_context!())
    .expect("error while running client");
}
