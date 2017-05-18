use std::env;
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
