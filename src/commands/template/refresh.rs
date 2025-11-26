use crate::cli::Args as CommonArgs;
use crate::commands::common;
use crate::commands::models::template::*;
use crate::commands::patterns::*;
use crate::filter::Filter;
use crate::path;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::fs::{read_to_string, write};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Parser)]
/// Refresh placeholder substitutions in files based on .gut/delta.toml
///
/// This command reads the replacements from .gut/delta.toml and applies them
/// to all files in the repository, replacing placeholders like __UND2C__ with
/// their actual values (e.g., 'se').
pub struct RefreshArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long)]
    /// Dry run - show what would be changed without actually changing files
    pub dry_run: bool,
    #[arg(long)]
    /// File pattern to refresh (e.g., "*.md", "src/**/*.rs")
    /// If not provided, all non-ignored files will be processed
    pub files: Option<Vec<String>>,
}

impl RefreshArgs {
    pub fn run(&self, _common_args: &CommonArgs) -> Result<()> {
        let root = common::root()?;
        let organisation = common::organisation(self.organisation.as_deref())?;
        let target_dirs =
            common::read_dirs_for_org(organisation.as_str(), &root, self.regex.as_ref())?;

        if target_dirs.is_empty() {
            println!(
                "No repositories found in organisation {} matching pattern {:?}",
                organisation, self.regex
            );
            return Ok(());
        }

        for dir in target_dirs {
            match refresh_repository(&dir, self.dry_run, self.files.as_ref()) {
                Ok(count) => {
                    if self.dry_run {
                        println!(
                            "✓ {:?}: Would refresh {} file(s) (dry run)",
                            path::dir_name(&dir)?,
                            count
                        );
                    } else {
                        println!(
                            "✓ {:?}: Refreshed {} file(s)",
                            path::dir_name(&dir)?,
                            count
                        );
                    }
                }
                Err(e) => println!("✗ {:?}: Failed - {:?}", path::dir_name(&dir), e),
            }
        }

        Ok(())
    }
}

fn refresh_repository(
    repo_dir: &PathBuf,
    dry_run: bool,
    file_patterns: Option<&Vec<String>>,
) -> Result<usize> {
    // Read delta.toml to get replacements
    let delta_path = repo_dir.join(".gut/delta.toml");
    if !delta_path.exists() {
        return Err(anyhow!(
            "No .gut/delta.toml found. This repository may not be based on a template."
        ));
    }

    let delta = TargetDelta::get(&delta_path)?;

    if delta.replacements.is_empty() {
        return Err(anyhow!("No replacements defined in .gut/delta.toml"));
    }

    // Get all files to process
    let files = get_files_to_refresh(repo_dir, file_patterns)?;

    let mut changed_count = 0;
    let mut error_count = 0;

    for file in files {
        let file_path = repo_dir.join(&file);

        // Process file and handle errors gracefully
        match process_file(&file_path, &file, &delta.replacements, dry_run) {
            Ok(true) => {
                changed_count += 1;
                println!("  ✓ {}", file);
            }
            Ok(false) => {
                // No changes needed, skip silently
            }
            Err(e) => {
                error_count += 1;
                eprintln!("  ✗ {}: {:?}", file, e);
            }
        }
    }

    if error_count > 0 {
        eprintln!(
            "  Warning: {} file(s) had errors and were skipped",
            error_count
        );
    }

    Ok(changed_count)
}

fn process_file(
    file_path: &PathBuf,
    file_name: &str,
    replacements: &std::collections::BTreeMap<String, String>,
    dry_run: bool,
) -> Result<bool> {
    // Skip if not a text file (binary files would be corrupted)
    if !is_text_file(file_path)? {
        return Ok(false);
    }

    // Read file content
    let content = read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_name))?;

    // Apply replacements
    let new_content = generate_string(replacements, &content)
        .with_context(|| format!("Failed to apply replacements to: {}", file_name))?;

    // Check if content changed
    if content != new_content {
        if !dry_run {
            write(file_path, new_content)
                .with_context(|| format!("Failed to write file: {}", file_name))?;
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

fn get_files_to_refresh(
    repo_dir: &PathBuf,
    file_patterns: Option<&Vec<String>>,
) -> Result<Vec<String>> {
    // Use git ls-files to respect .gitignore
    let output = Command::new("git")
        .arg("ls-files")
        .current_dir(repo_dir)
        .output()
        .context("Failed to run git ls-files. Is this a git repository?")?;

    if !output.status.success() {
        return Err(anyhow!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let files_output = String::from_utf8_lossy(&output.stdout);
    let all_files: Vec<String> = files_output
        .lines()
        .map(|s| s.to_string())
        .filter(|f| {
            // Skip .gut directory files
            !f.starts_with(".gut/") && f != ".gut"
        })
        .collect();

    // If specific patterns provided, filter further
    if let Some(patterns) = file_patterns {
        let mut matched = Vec::new();
        for pattern in patterns {
            for file in &all_files {
                if file_matches_pattern(file, pattern) {
                    if !matched.contains(file) {
                        matched.push(file.clone());
                    }
                }
            }
        }
        Ok(matched)
    } else {
        Ok(all_files)
    }
}

fn file_matches_pattern(file: &str, pattern: &str) -> bool {
    // Simple glob matching
    if pattern.contains('*') {
        let regex_pattern = pattern
            .replace(".", "\\.")
            .replace("**", ".*")
            .replace("*", "[^/]*");
        if let Ok(re) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
            return re.is_match(file);
        }
    }
    file == pattern || file.ends_with(pattern)
}

fn is_text_file(path: &PathBuf) -> Result<bool> {
    // Check if file extension suggests it's a text file
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        let text_extensions = vec![
            "txt", "md", "rs", "toml", "yaml", "yml", "json", "xml", "html", "css", "js", "ts",
            "py", "sh", "bash", "zsh", "fish", "c", "h", "cpp", "hpp", "java", "kt", "swift",
            "go", "rb", "php", "pl", "r", "tex", "bib", "gitignore", "gitattributes", "config",
            "cfg", "ini", "conf", "dockerfile", "makefile", "cmake", "editorconfig",
        ];

        if text_extensions.contains(&ext.as_ref()) {
            return Ok(true);
        }
    }

    // Files without extension that are typically text
    if let Some(name) = path.file_name() {
        let name = name.to_string_lossy().to_lowercase();
        let text_names = vec![
            "readme",
            "license",
            "changelog",
            "makefile",
            "dockerfile",
            "gitignore",
            "gitattributes",
        ];

        if text_names.contains(&name.as_ref()) {
            return Ok(true);
        }
    }

    // Default to false for unknown types to avoid corrupting binary files
    Ok(false)
}
