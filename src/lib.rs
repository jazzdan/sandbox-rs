use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

pub trait CommandExecutor {
    fn execute(
        &self,
        command: &[String],
        files: &[impl AsRef<Path>],
        working_dir: &Path,
    ) -> Result<()>;
}

pub struct LinuxCommandExecutor;

impl LinuxCommandExecutor {
    pub fn new() -> Self {
        LinuxCommandExecutor
    }
}

impl CommandExecutor for LinuxCommandExecutor {
    fn execute(
        &self,
        command: &[String],
        files: &[impl AsRef<Path>],
        working_dir: &Path,
    ) -> Result<()> {
        // Create a temporary directory
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;

        // Copy all specified files to the temporary directory, maintaining directory structure
        for file in files {
            let file_path = file.as_ref();
            let relative_path = file_path.strip_prefix(working_dir).with_context(|| {
                format!("File {:?} is not within the working directory", file_path)
            })?;
            let dest_path = temp_dir.path().join(relative_path);

            // Create parent directories if they don't exist
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {:?}", parent))?;
            }

            fs::copy(file_path, &dest_path)
                .with_context(|| format!("Failed to copy file: {:?}", file_path))?;
        }

        // Prepare the command
        let mut cmd = Command::new(&command[0]);
        cmd.args(&command[1..]).current_dir(temp_dir.path());

        // Execute the command
        let output = cmd.output().context("Failed to execute command")?;

        // Check if the command was successful
        if output.status.success() {
            println!("Command executed successfully");
            println!("Output: {}", String::from_utf8_lossy(&output.stdout));
            Ok(())
        } else {
            let error_message = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command failed: {}", error_message)
        }
    }
}
