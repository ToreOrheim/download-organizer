use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;

fn main() {
    let folder_path = env::var("DOWNLOAD_FOLDER_PATH").expect("Environment variable for 'DOWNLOAD_FOLDER_PATH' not set");

    // Spawn thread to watch folder path
    let (sender, receiver) = channel();
    spawn_watcher(folder_path.to_string(), sender);

    // Handle event loop
    loop {
        // If our channel receives a value
        match receiver.recv() {
            Ok(event) => {
                match event {
                    Event::Added(file_path) => {
                        // Determine the extension of the file
                        let extension = file_path
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .unwrap_or("");

                        // Define the directory based on the extension
                        let subdirectory = match extension {
                            "txt" => "text_files",
                            "jpg" | "jpeg" | "png" => "image_files",
                            "zip" => "zip_files",
                            _ => "other_files",
                        };

                        // Create the directory if it doesn't exist
                        let subdirectory_path = format!("{}/{}", folder_path, subdirectory);
                        if !Path::new(&subdirectory_path).exists() {
                            fs::create_dir(&subdirectory_path).expect("Failed to create subdirectory");
                        }

                        // Move the file
                        let new_file_path = format!("{}/{}", subdirectory_path, file_path.file_name().unwrap().to_str().unwrap());
                        fs::rename(&file_path, &new_file_path).expect("Failed to move file");
                        println!("Moved {} to {}", file_path.display(), new_file_path);
                    }
                }
            }
            // Yikes hard pass
            Err(e) => {
                eprintln!("Error: {:?}", e);
            }
        }
    }
}

// Custom event enum
enum Event {
    Added(PathBuf),
}

// Spawn a thread that will watch for changes
fn spawn_watcher(folder_path: String, sender: Sender<Event>) {
    thread::spawn(move || {
        let mut previous_files = get_files(&folder_path).unwrap();

        // Main event loop
        loop {
            thread::sleep(Duration::from_secs(10));

            if let Ok(files) = get_files(&folder_path) {
                let new_files: Vec<_> = files.iter().filter(|f| !previous_files.contains(f)).collect();

                for file in new_files {
                    sender.send(Event::Added(file.clone())).expect("Failed to send event");
                }

                previous_files = files;
            }
        }
    });
}

// Get a list of files in folder_path 
fn get_files(folder_path: &str) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }

    Ok(files)
}

