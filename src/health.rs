use colored::*;
use std::process::Command;

/// Health check warnings that should be displayed to the user
#[derive(Debug, Clone)]
pub struct HealthWarning {
    pub title: String,
    pub message: String,
    pub suggestion: Option<String>,
}

impl HealthWarning {
    pub fn print(&self) {
        eprintln!("\n{} {}", "⚠️  Warning:".yellow().bold(), self.title.yellow());
        eprintln!("   {}", self.message);
        if let Some(suggestion) = &self.suggestion {
            eprintln!("   {}", suggestion.bright_black());
        }
    }
}

/// Run health checks and return any warnings
pub fn check_git_config() -> Vec<HealthWarning> {
    let mut warnings = Vec::new();

    // Check core.precomposeUnicode on macOS
    if cfg!(target_os = "macos") {
        if let Some(warning) = check_precompose_unicode() {
            warnings.push(warning);
        }
    }

    // Check if Git LFS is installed
    if let Some(warning) = check_git_lfs_installed() {
        warnings.push(warning);
    }

    warnings
}

/// Check if core.precomposeUnicode is properly set on macOS
fn check_precompose_unicode() -> Option<HealthWarning> {
    let output = Command::new("git")
        .args(&["config", "--get", "core.precomposeUnicode"])
        .output()
        .ok()?;

    let value = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();

    if value != "true" {
        return Some(HealthWarning {
            title: "core.precomposeUnicode not set".to_string(),
            message: "Git setting 'core.precomposeUnicode' is not enabled.".to_string(),
            suggestion: Some(
                "Run: git config --global core.precomposeUnicode true\n   \
                This prevents Unicode normalization issues with filenames on macOS."
                    .to_string(),
            ),
        });
    }

    None
}

/// Check if Git LFS is installed
fn check_git_lfs_installed() -> Option<HealthWarning> {
    let output = Command::new("git")
        .args(&["lfs", "version"])
        .output()
        .ok()?;

    if !output.status.success() {
        return Some(HealthWarning {
            title: "Git LFS not installed".to_string(),
            message: "Git LFS is not installed or not properly configured.".to_string(),
            suggestion: Some(
                "Many repositories use Git LFS for large files. Install it with:\n   \
                macOS: brew install git-lfs && git lfs install\n   \
                Linux: apt-get install git-lfs && git lfs install (or equivalent for your distro)"
                    .to_string(),
            ),
        });
    }

    None
}

/// Print all health warnings at the end of command execution
pub fn print_warnings(warnings: &[HealthWarning]) {
    if !warnings.is_empty() {
        eprintln!(); // Empty line before warnings
        for warning in warnings {
            warning.print();
        }
    }
}

// Additional health checks that could be implemented:
//
// 1. ✅ Check for LFS installation when repo uses Git LFS
// 2. Check for sufficient disk space
// 3. Check Git version (minimum required version)
// 4. Check for proper SSH key configuration
// 5. Check for .gitignore patterns that might cause issues
// 6. Check for very long filenames (macOS has limits)
// 7. ✅ Check for case sensitivity issues (macOS is case-insensitive by default)
// 8. Check for proper line ending configuration (core.autocrlf)
// 9. Check for Git credential helper configuration
// 10. ✅ Check for NFD/NFC normalization conflicts in existing repos
// 11. ✅ Check for case-duplicate filenames (identical names except for letter case)
// 10. Check for NFD/NFC normalization conflicts in existing repos
