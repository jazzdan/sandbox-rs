use anyhow::Result;
use sandbox_rs::{CommandExecutor, LinuxCommandExecutor};
use std::env;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let current_dir = env::current_dir()?;
    let executor = LinuxCommandExecutor::new(&current_dir);

    let this_file = Path::new(std::file!());
    let full_file_path = current_dir.join(this_file);
    let command = vec!["cat".to_string(), this_file.to_str().unwrap().to_string()];
    let files = vec![full_file_path];

    executor.execute(&command, &files)
}
