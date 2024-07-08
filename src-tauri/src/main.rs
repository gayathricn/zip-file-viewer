use std::fs::{File, OpenOptions};
use std::sync::Mutex;
use zip::ZipArchive;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use lazy_static::lazy_static;
use tauri::Manager;

const RECENT_FILES_PATH: &str = "recent_files.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RecentFile {
    path: String,
    timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ZipContent {
    path: String,
}

lazy_static! {
    static ref RECENT_FILES: Mutex<Vec<RecentFile>> = Mutex::new(read_recent_files().unwrap_or_else(|_| Vec::new()));
}

#[tauri::command]
fn list_contents(zip_file: String, password: Option<String>) -> Result<Vec<ZipContent>, String> {
    println!("Listing contents of ZIP file at path: {}", zip_file);
    let file = File::open(&zip_file).map_err(|e| format!("Failed to open ZIP file: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to parse ZIP file: {}", e))?;

    let mut contents = Vec::new();
    let password_bytes: Option<&[u8]> = password.as_deref().map(|s| s.as_bytes());

    for i in 0..archive.len() {
        let file_result = match password_bytes {
            Some(p) => archive.by_index_decrypt(i, p).map_err(|e| format!("Error reading file in ZIP: {}", e)).and_then(|r| r.map_err(|e| format!("Invalid password: {}", e))),
            None => archive.by_index(i).map_err(|e| format!("Error reading file in ZIP: {}", e)),
        };

        let file = file_result?;
        contents.push(ZipContent {
            path: file.name().to_string(),
        });
    }

    update_recent_files(&zip_file)?;

    Ok(contents)
}

#[tauri::command]
fn get_recent_files() -> Result<Vec<RecentFile>, String> {
    let recent_files = RECENT_FILES.lock().unwrap().clone();
    Ok(recent_files)
}

#[tauri::command]
fn add_recent_file(file_path: String) -> Result<(), String> {
    update_recent_files(&file_path)
}

fn update_recent_files(file_path: &str) -> Result<(), String> {
    let mut recent_files = RECENT_FILES.lock().unwrap();
    recent_files.retain(|file| file.path != file_path);

    recent_files.push(RecentFile {
        path: file_path.to_string(),
        timestamp: Utc::now().timestamp_millis() as u64,
    });

    if recent_files.len() > 5 {
        recent_files.remove(0);
    }

    Ok(())
}

fn read_recent_files() -> Result<Vec<RecentFile>, String> {
    let file = File::open(RECENT_FILES_PATH).map_err(|_| "No recent files found".to_string())?;
    let recent_files: Vec<RecentFile> = serde_json::from_reader(file).map_err(|e| format!("Failed to parse recent files: {}", e))?;
    Ok(recent_files)
}

fn write_recent_files() -> Result<(), String> {
    let recent_files = RECENT_FILES.lock().unwrap();
    let file = OpenOptions::new().create(true).write(true).truncate(true).open(RECENT_FILES_PATH)
        .map_err(|e| format!("Failed to open recent files file: {}", e))?;
    serde_json::to_writer(file, &*recent_files).map_err(|e| format!("Failed to write recent files: {}", e))?;
    Ok(())
}

fn write_recent_files_on_exit() -> Result<(), String> {
    write_recent_files()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![list_contents, get_recent_files, add_recent_file])
        .setup(|app| {
            let handle = app.handle();
            app.listen_global("tauri://close-requested", move |_event| {
                handle.exit(0);
            });
            Ok(())
        })
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                if let Err(e) = write_recent_files_on_exit() {
                    eprintln!("Failed to write recent files on exit: {}", e);
                }
                api.prevent_close();
                std::process::exit(0);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
