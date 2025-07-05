// This file is for random tests.

use std::io::Read;

#[path = "../config.rs"]
mod config;
#[path = "../dirs.rs"]
mod dirs;

fn main() {
    zzz();
}

fn zzz() {
    let files = vec![
        ("spike", "submerged.llsp3"),
        ("spike", "Project 1.llsp3"),
        ("spike", "Iconic.llsp3"),
        ("mindstorms", "Project 1.lms"),
        ("mindstorms", "line follower.lms"),
    ];

    let cfg = config::Config::load(None).unwrap();
    let dirs = dirs::Dirs::new(&cfg).unwrap();

    for (dir_type, filename) in files {
        let dir = match dir_type {
            "spike" => &dirs.spike,
            "mindstorms" => &dirs.mindstorms,
            _ => continue,
        };
        let path = dir.join(filename);
        describe_file(&path, dir_type);
    }
}

fn describe_file(file_path: &std::path::Path, dir_type: &str) {
    if !file_path.exists() {
        println!(
            "{}: {} - File not found",
            dir_type,
            file_path.file_name().unwrap().to_string_lossy()
        );
        return;
    }

    let file_name = file_path.file_name().unwrap().to_string_lossy();
    println!("{}: {}", dir_type, file_name);

    // Read file header to determine type
    let mut file = match std::fs::File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            println!("  Error opening file: {}", e);
            return;
        }
    };

    let mut header = [0u8; 8];
    if let Err(e) = file.read_exact(&mut header) {
        println!("  Error reading file header: {}", e);
        return;
    }

    // Check file type
    if header.starts_with(b"PK") {
        println!("  Type: ZIP file");
        describe_zip_contents(file_path);
    } else if header.starts_with(b"{") || header.starts_with(b"[") {
        println!("  Type: JSON file");
    } else {
        println!("  Type: Other (header: {:?})", header);
    }
}

fn describe_zip_contents(zip_path: &std::path::Path) {
    let file = match std::fs::File::open(zip_path) {
        Ok(f) => f,
        Err(e) => {
            println!("    Error opening ZIP: {}", e);
            return;
        }
    };

    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => {
            println!("    Error reading ZIP: {}", e);
            return;
        }
    };

    println!("    Contents:");
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name();
            println!("      {}", name);

            // Check if this is a nested ZIP file
            if name.ends_with(".zip") || name.ends_with(".llsp3") || name.ends_with(".lms") {
                println!("        (potential nested archive)");
            }
        }
    }
}
