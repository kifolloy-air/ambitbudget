// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use serde::Serialize;

// All Ambit data lives under ~/Documents/Ambit/ — visible, copyable, syncable.
fn data_root() -> PathBuf {
    dirs::document_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Ambit")
}
fn receipts_root() -> PathBuf { data_root().join("receipts") }
fn data_file()    -> PathBuf { data_root().join("data.json") }

// Reject filenames containing any of these characters to avoid path traversal
// or hitting Windows reserved chars.
fn _is_unsafe_id(id: &str) -> bool {
    id.is_empty()
        || id.contains('/')
        || id.contains('\\')
        || id.contains(':')
        || id.contains('*')
        || id.contains('?')
        || id.contains('"')
        || id.contains('<')
        || id.contains('>')
        || id.contains('|')
}

#[derive(Serialize)]
struct PathInfo {
    data_dir: String,
    data_file: String,
    receipts_dir: String,
}

#[tauri::command]
fn ambit_paths() -> PathInfo {
    PathInfo {
        data_dir: data_root().to_string_lossy().into_owned(),
        data_file: data_file().to_string_lossy().into_owned(),
        receipts_dir: receipts_root().to_string_lossy().into_owned(),
    }
}

#[tauri::command]
fn ambit_read_data() -> Result<String, String> {
    let p = data_file();
    if !p.exists() { return Ok(String::new()); }
    fs::read_to_string(&p).map_err(|e| format!("read {}: {}", p.display(), e))
}

#[tauri::command]
fn ambit_write_data(content: String) -> Result<(), String> {
    // Atomic write: temp file -> rotate previous version to .bak -> rename
    // temp -> live. Crash mid-write leaves an orphaned .tmp; the live
    // data.json is never half-written. The .bak is the previous good copy
    // so we can recover if the new write was bad logic.
    let dir = data_root();
    fs::create_dir_all(&dir).map_err(|e| format!("mkdir {}: {}", dir.display(), e))?;

    let target = data_file();
    let tmp = target.with_extension("json.tmp");
    let backup = target.with_extension("json.bak");

    fs::write(&tmp, content).map_err(|e| format!("write tmp: {}", e))?;

    if target.exists() {
        if backup.exists() {
            let _ = fs::remove_file(&backup);
        }
        fs::rename(&target, &backup).map_err(|e| format!("rotate backup: {}", e))?;
    }

    fs::rename(&tmp, &target).map_err(|e| format!("commit write: {}", e))?;
    Ok(())
}

#[tauri::command]
fn ambit_save_receipt(id: String, bytes: Vec<u8>) -> Result<String, String> {
    if _is_unsafe_id(&id) { return Err("invalid receipt id".into()); }
    let dir = receipts_root();
    fs::create_dir_all(&dir).map_err(|e| format!("mkdir receipts: {}", e))?;
    let p = dir.join(format!("{}.bin", id));
    fs::write(&p, bytes).map_err(|e| format!("write receipt: {}", e))?;
    Ok(p.to_string_lossy().into_owned())
}

#[tauri::command]
fn ambit_load_receipt(id: String) -> Result<Vec<u8>, String> {
    if _is_unsafe_id(&id) { return Err("invalid receipt id".into()); }
    let p = receipts_root().join(format!("{}.bin", id));
    if !p.exists() { return Err("not found".into()); }
    fs::read(&p).map_err(|e| format!("read receipt: {}", e))
}

#[tauri::command]
fn ambit_delete_receipt(id: String) -> Result<(), String> {
    if _is_unsafe_id(&id) { return Err("invalid receipt id".into()); }
    let p = receipts_root().join(format!("{}.bin", id));
    if p.exists() {
        fs::remove_file(&p).map_err(|e| format!("delete receipt: {}", e))?;
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        // Single-instance plugin: if the user double-clicks the launcher,
        // focus the existing window instead of opening a second process
        // (which would race on data.json writes and lose data).
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            ambit_paths,
            ambit_read_data,
            ambit_write_data,
            ambit_save_receipt,
            ambit_load_receipt,
            ambit_delete_receipt,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
