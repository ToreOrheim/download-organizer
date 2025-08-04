use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

fn main() {
    let folder_path = env::var("DOWNLOAD_FOLDER_PATH")
        .or_else(|_| env::var("XDG_DOWNLOAD_DIR"))
        .expect("Environment variable for 'DOWNLOAD_FOLDER_PATH' not set");

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
                            "pdf" => "pdfs",
                            "csv" | "xlms" => "data_files",
                            "exe" => "executables",
                            "deb" => "debian_package_files",
                            "appimage" => "appimages",
                            _ => "other_files",
                        };

                        // Create the directory if it doesn't exist
                        let subdirectory_path = format!("{}/{}", folder_path, subdirectory);
                        if !Path::new(&subdirectory_path).exists() {
                            fs::create_dir(&subdirectory_path)
                                .expect("Failed to create subdirectory");
                        }

                        // Move the file
                        let new_file_path = format!(
                            "{}/{}",
                            subdirectory_path,
                            file_path.file_name().unwrap().to_str().unwrap()
                        );
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
    println!("Watcher thread started on folder: {}", folder_path);

    thread::spawn(move || {
        let mut previous_files = match get_files(&folder_path) {
            Ok(files) => files,
            Err(err) => {
                eprintln!("Failed to read folder on startup: {}", err);
                return;
            }
        };

        // Main event loop
        loop {
            thread::sleep(Duration::from_secs(10));

            if let Ok(files) = get_files(&folder_path) {
                let new_files: Vec<_> = files
                    .iter()
                    .filter(|f| !previous_files.contains(f))
                    .collect();

                for file in new_files {
                    println!("New files detected: {:?}", file.display());
                    if let Err(err) = sender.send(Event::Added(file.clone())) {
                        eprintln!("Failed to send event: {}", err);
                    }
                }

                previous_files = files;
            } else {
                eprintln!("Failed to read folder: {}", folder_path)
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
