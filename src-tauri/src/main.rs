// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use serde::Serialize;

// Live Ambit data lives in the OS-managed app-data location:
//   Windows: %LOCALAPPDATA%\Ambit                          (C:\Users\<u>\AppData\Local\Ambit)
//   macOS:   ~/Library/Application Support/Ambit
//   Linux:   ~/.local/share/Ambit                           (XDG_DATA_HOME)
// This is where serious desktop apps put working data: out of the way of
// the user, out of cloud-sync paths that can corrupt files mid-write,
// and not affected by OneDrive/iCloud Documents redirection (which
// caused real confusion during earlier testing).
//
// User-controlled backups still go wherever the user wants them via the
// existing Export Backup / Send to Device flows in the app.
fn data_root() -> PathBuf {
    dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Ambit")
}
fn receipts_root() -> PathBuf { data_root().join("receipts") }
fn data_file()    -> PathBuf { data_root().join("data.json") }

// One-time migration from the old ~/Documents/Ambit location used by
// builds prior to this change. Runs at most once per machine: if the new
// data file doesn't exist but the old one does, copy the old file (plus
// .bak and the receipts subfolder if present) into the new location.
// The old folder is left in place as a safety backup; the user can
// delete it manually once they're confident the migration took.
fn try_migrate_from_documents() {
    let new_data = data_file();
    if new_data.exists() {
        return; // already on the new location
    }
    let old_root = match dirs::document_dir() {
        Some(d) => d.join("Ambit"),
        None => return,
    };
    let old_data = old_root.join("data.json");
    if !old_data.exists() {
        return; // nothing to migrate
    }
    let new_dir = data_root();
    if fs::create_dir_all(&new_dir).is_err() { return; }
    let _ = fs::copy(&old_data, &new_data);
    let old_bak = old_root.join("data.json.bak");
    if old_bak.exists() {
        let _ = fs::copy(&old_bak, new_dir.join("data.json.bak"));
    }
    let old_receipts = old_root.join("receipts");
    if old_receipts.is_dir() {
        let new_receipts = receipts_root();
        let _ = fs::create_dir_all(&new_receipts);
        if let Ok(entries) = fs::read_dir(&old_receipts) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                        let _ = fs::copy(&p, new_receipts.join(name));
                    }
                }
            }
        }
    }
}

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
    if !p.exists() {
        // First read after the AppData relocation. Try to lift the user's
        // existing data from the old Documents/Ambit folder before treating
        // this as a fresh install.
        try_migrate_from_documents();
    }
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
