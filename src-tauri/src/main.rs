use std::fs::{File, OpenOptions};
use std::sync::Mutex;
use zip::ZipArchive;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use lazy_static::lazy_static;

const RECENT_FILES_PATH: &str = "recent_files.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RecentFile {
    path: String,
    timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ZipContent {
    name: String,
    // encrypted: bool, // Remove this if not used
}

lazy_static! {
    static ref RECENT_FILES: Mutex<Vec<RecentFile>> = Mutex::new(Vec::new());
}

// Tauri command to list contents (file names) of a ZIP file
#[tauri::command]
fn list_contents(zip_file: String, password: Option<String>) -> Result<Vec<ZipContent>, String> {
    println!("Listing contents of ZIP file at path: {}", zip_file);

    // Log the path received
    println!("Received zip_file path: {}", zip_file);

    // Open the ZIP file
    let file = File::open(&zip_file).map_err(|e| format!("Failed to open ZIP file: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to parse ZIP file: {}", e))?;

    let mut contents = Vec::new();
    // Convert password to Option<&[u8]>
    let password_bytes: Option<&[u8]> = password.as_deref().map(|s| s.as_bytes());

    // Iterate through each file in the ZIP archive and collect their names
    for i in 0..archive.len() {
        let file_result = match password_bytes {
            Some(p) => archive.by_index_decrypt(i, p).map_err(|e| format!("Error reading file in ZIP: {}", e)).and_then(|r| r.map_err(|e| format!("Invalid password: {}", e))),
            None => archive.by_index(i).map_err(|e| format!("Error reading file in ZIP: {}", e)),
        };

        let file = file_result?;

        contents.push(ZipContent {
            name: file.name().to_string(),
            // encrypted: file.is_encrypted(), // Remove this if not used
        });
    }

    add_recent_file_to_memory(&zip_file)?;

    Ok(contents)
}

// Tauri command to get recent files
#[tauri::command]
fn get_recent_files() -> Result<Vec<RecentFile>, String> {
    let recent_files = read_recent_files()?;
    Ok(recent_files)
}

// Tauri command to add a recent file to the list
#[tauri::command]
fn add_recent_file(file_path: String) -> Result<(), String> {
    add_recent_file_to_memory(&file_path)
}

// Helper function to add a file to the recent files list in memory
fn add_recent_file_to_memory(file_path: &str) -> Result<(), String> {
    let mut recent_files = RECENT_FILES.lock().unwrap();

    recent_files.retain(|file| file.path != file_path); // Remove existing entry for the file if exists

    recent_files.push(RecentFile {
        path: file_path.to_string(),
        timestamp: Utc::now().timestamp_millis() as u64,
    });

    if recent_files.len() > 5 {
        recent_files.remove(0); // Keep only the last 5 entries
    }

    Ok(())
}

// Helper function to read the recent files list from the file
fn read_recent_files() -> Result<Vec<RecentFile>, String> {
    if cfg!(debug_assertions) {
        // In development mode, read from the in-memory list
        Ok(RECENT_FILES.lock().unwrap().clone())
    } else {
        // In production mode, read from the file
        let file = File::open(RECENT_FILES_PATH).map_err(|_| "No recent files found".to_string())?;
        let recent_files: Vec<RecentFile> = serde_json::from_reader(file).map_err(|e| format!("Failed to parse recent files: {}", e))?;
        Ok(recent_files)
    }
}

// Helper function to write the recent files list to the file
fn write_recent_files(recent_files: &[RecentFile]) -> Result<(), String> {
    if cfg!(debug_assertions) {
        // In development mode, do nothing
        Ok(())
    } else {
        // In production mode, write to the file
        let file = OpenOptions::new().create(true).write(true).truncate(true).open(RECENT_FILES_PATH)
            .map_err(|e| format!("Failed to open recent files file: {}", e))?;
        serde_json::to_writer(file, &recent_files).map_err(|e| format!("Failed to write recent files: {}", e))?;
        Ok(())
    }
}

// Helper function to write recent files from memory to the file on exit
fn write_recent_files_on_exit() -> Result<(), String> {
    let recent_files = RECENT_FILES.lock().unwrap();
    write_recent_files(&recent_files)
}

fn main() {
    // Initialize Tauri application builder
    tauri::Builder::default()
        // Register Tauri commands
        .invoke_handler(tauri::generate_handler![list_contents, get_recent_files, add_recent_file])
        // Hook into the application exit event using a Tauri plugin or a custom implementation
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, e| {
            if let tauri::RunEvent::ExitRequested { api, .. } = e {
                if let Err(err) = write_recent_files_on_exit() {
                    eprintln!("Failed to write recent files on exit: {}", err);
                }
                api.prevent_exit();
            }
        });
}
