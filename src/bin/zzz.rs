// This file is for random tests.

use std::{
    io::{Cursor, Read},
    path::Path,
};

#[path = "../config.rs"]
#[allow(dead_code)]
mod config;
#[path = "../dirs.rs"]
#[allow(dead_code)]
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
        println!("{dir_type}/{filename}");
        let dir = match dir_type {
            "spike" => &dirs.spike,
            "mindstorms" => &dirs.mindstorms,
            _ => continue,
        };
        let path = dir.join(filename);
        describe_file(&path, "  ");
    }
}

fn describe_file(path: &Path, indent: &str) {
    match std::fs::read(path) {
        Err(e) => println!("{indent}error: {e}"),
        Ok(contents) => describe_file_contents(&contents, indent, false),
    };
}

fn describe_file_contents(contents: &[u8], indent: &str, show_raw: bool) {
    if contents.is_empty() {
        println!("{indent}file type: empty");
        return;
    }

    if show_raw {
        println!("{}", String::from_utf8_lossy(contents));
    }

    if contents.starts_with(b"PK") {
        println!("{indent}file type: zip");
        describe_zip(contents, indent);
        return;
    }

    if contents.starts_with(b"<svg") {
        println!("{indent}file type: svg");
        return;
    }

    // The examples I have start with ff fb 90 64. 'file' identifies this as "MPEG ADTS, layer III,
    // v1", etc. The docs I've found say that there is supposed to be 12x 1s, then a 0 or 1, then
    // 2x 0s, but this (ff fb) is 12x 1s, 1x 1, 10. So it's not right, but scratch doesn't seem to
    // care. So we'll just call 12x 1s good enough.
    if contents.len() > 2 && contents[0] == 0xff && contents[1] & 0xf0 == 0xf0 {
        println!("{indent}file type: MPEG");
        return;
    }

    if serde_json::from_slice::<serde_json::Value>(contents).is_ok() {
        println!("{indent}file type: json");
        return;
    }

    let magic = if contents.len() > 8 {
        &contents[..8]
    } else {
        contents
    };
    let magicstr = String::from_utf8_lossy(magic);
    println!("{indent}file type: unknown ({magic:x?} / {magicstr:?})");
}

fn describe_zip(contents: &[u8], indent: &str) {
    let mut reader = zip::ZipArchive::new(Cursor::new(contents)).unwrap();
    let subindent = format!("{indent}  ");
    for i in 0..reader.len() {
        let mut file = reader.by_index(i).unwrap();
        let name = file.name();
        let show_raw = show_raw(name);
        println!("{indent}+ file name: {name}");
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();
        describe_file_contents(&contents, &subindent, show_raw);
    }
}

fn show_raw(filename: &str) -> bool {
    filename == "nothing" // Change this to a file you want to see the contents of.
}
