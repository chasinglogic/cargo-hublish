use std::env;
use std::process;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

pub fn get_project_root() -> Option<PathBuf> {
    let mut cwd = env::current_dir().unwrap();

    loop {
        cwd.push("Cargo.toml");

        if cwd.exists() {
            cwd.pop();
            return Some(cwd);
        }

        cwd.pop();
        if !cwd.pop() {
            return None;
        }
    }
}


pub fn read_file(p: &Path) -> String {
    let mut file = File::open(p).unwrap();
    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Ok(_) => content,
        Err(e) => { println!("Error: {}", e); process::exit(1) },
    }
}
