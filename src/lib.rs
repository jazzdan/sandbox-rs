use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

pub trait CommandExecutor {
    fn execute<P: AsRef<Path>>(
        &self,
        command: &[String],
        files: &[P],
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
    fn execute<P: AsRef<Path>>(
        &self,
        command: &[String],
        files: &[P],
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_linux_command_executor_file_access() {
        // 1. Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // 2. Create a file in that temp directory
        let file_name = "test_file.txt";
        let file_content = "This is a test file";
        let file_path = temp_path.join(file_name);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{}", file_content).unwrap();

        // 3. Find the absolute path to that file
        let absolute_file_path = file_path.canonicalize().unwrap();

        // 4. Run a command in the sandbox that reads that absolute path
        let executor = LinuxCommandExecutor::new();
        let command = vec![
            "cat".to_string(),
            absolute_file_path.to_str().unwrap().to_string(),
        ];

        // 5. Expect the command to fail (but it will actually succeed)
        let result = executor.execute::<std::path::PathBuf>(&command, &[], temp_path);

        // This assertion will fail because the command succeeds
        assert!(
            result.is_err(),
            "Expected command to fail, but it succeeded"
        );
    }
}
