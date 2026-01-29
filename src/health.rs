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

    // Check Git version
    if let Some(warning) = check_git_version() {
        warnings.push(warning);
    }

    // Check core.precomposeUnicode on macOS
    if cfg!(target_os = "macos") {
        if let Some(warning) = check_precompose_unicode() {
            warnings.push(warning);
        }
    }

    // Check core.autocrlf on Unix systems (macOS/Linux)
    if cfg!(unix) {
        if let Some(warning) = check_autocrlf() {
            warnings.push(warning);
        }
    }

    // Check if Git LFS is installed
    if let Some(warning) = check_git_lfs_installed() {
        warnings.push(warning);
    }

    warnings
}

/// Check if Git version meets minimum requirements (>= 1.7.10)
fn check_git_version() -> Option<HealthWarning> {
    let output = Command::new("git")
        .args(&["--version"])
        .output()
        .ok()?;

    let version_output = String::from_utf8_lossy(&output.stdout);
    
    // Parse version from output like "git version 2.39.3 (Apple Git-146)"
    let version_str = version_output
        .split_whitespace()
        .nth(2)?;
    
    let parts: Vec<&str> = version_str.split('.').collect();
    if parts.len() >= 2 {
        let major = parts[0].parse::<u32>().ok()?;
        let minor = parts[1].parse::<u32>().ok()?;
        
        // Require Git >= 1.7.10
        // Check: version is 1.7.x where x < 10, or version is 1.x where x < 7, or version is 0.x
        if major < 1 || (major == 1 && minor < 7) {
            return Some(HealthWarning {
                title: "Git version too old".to_string(),
                message: format!("Git version {}.{} is too old. Minimum required is 1.7.10.", major, minor),
                suggestion: Some(
                    "Update Git to version 1.7.10 or newer:\n   \
                    macOS: brew upgrade git\n   \
                    Linux: Use your distribution's package manager to update git"
                        .to_string(),
                ),
            });
        } else if major == 1 && minor == 7 {
            // For 1.7.x, check the patch version
            if parts.len() >= 3 {
                let patch = parts[2].parse::<u32>().ok().unwrap_or(0);
                if patch < 10 {
                    return Some(HealthWarning {
                        title: "Git version too old".to_string(),
                        message: format!("Git version {}.{}.{} is too old. Minimum required is 1.7.10.", major, minor, patch),
                        suggestion: Some(
                            "Update Git to version 1.7.10 or newer:\n   \
                            macOS: brew upgrade git\n   \
                            Linux: Use your distribution's package manager to update git"
                                .to_string(),
                        ),
                    });
                }
            }
        }
    }

    None
}

/// Check if core.precomposeUnicode is properly set on macOS
/// 
/// Since Git 1.7.10, the default behavior on macOS is to use precomposed Unicode (NFC),
/// so it's OK if this setting is not explicitly set. We only warn if it's explicitly
/// set to false.
fn check_precompose_unicode() -> Option<HealthWarning> {
    let output = Command::new("git")
        .args(&["config", "--get", "core.precomposeUnicode"])
        .output()
        .ok()?;

    let value = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();

    // Only warn if explicitly set to false (empty/unset is OK as default is true since Git 1.7.10)
    if value == "false" {
        return Some(HealthWarning {
            title: "core.precomposeUnicode disabled".to_string(),
            message: "Git setting 'core.precomposeUnicode' is explicitly disabled.".to_string(),
            suggestion: Some(
                "Run: git config --global --unset core.precomposeUnicode\n   \
                Or: git config --global core.precomposeUnicode true\n   \
                This prevents Unicode normalization issues with filenames on macOS."
                    .to_string(),
            ),
        });
    }

    None
}

/// Check if core.autocrlf is properly set on Unix systems (macOS/Linux)
/// 
/// Having core.autocrlf=true on Unix systems can cause problems:
/// - Automatic CRLF conversion can corrupt binary files
/// - Can cause Git to report changes that don't actually exist
/// - Is user-specific rather than repository-specific
/// Best practice: Use .gitattributes files in repositories instead
fn check_autocrlf() -> Option<HealthWarning> {
    let output = Command::new("git")
        .args(&["config", "--get", "core.autocrlf"])
        .output()
        .ok()?;

    let value = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();

    // Warn if set to "true" on Unix systems (problematic)
    // "false", "input", or empty (unset) are all OK
    if value == "true" {
        return Some(HealthWarning {
            title: "core.autocrlf enabled on Unix system".to_string(),
            message: "Git setting 'core.autocrlf' is set to 'true', which can cause problems on Unix systems.".to_string(),
            suggestion: Some(
                "Run: git config --global core.autocrlf input\n   \
                Or: git config --global --unset core.autocrlf\n   \
                \n   \
                Best practice: Use .gitattributes files in your repositories instead.\n   \
                Example .gitattributes content:\n   \
                  * text=auto\n   \
                  *.sh text eol=lf\n   \
                  *.bat text eol=crlf\n   \
                \n   \
                This ensures consistent behavior across all team members."
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

/// Get the current Git version as a string
pub fn get_git_version() -> Option<String> {
    let output = Command::new("git")
        .args(&["--version"])
        .output()
        .ok()?;

    let version_output = String::from_utf8_lossy(&output.stdout);
    
    // Parse version from output like "git version 2.39.3 (Apple Git-146)"
    version_output
        .split_whitespace()
        .nth(2)
        .map(|v| v.to_string())
}

/// Get the current core.precomposeUnicode setting value
pub fn get_precompose_unicode_value() -> String {
    let output = Command::new("git")
        .args(&["config", "--get", "core.precomposeUnicode"])
        .output();
    
    match output {
        Ok(out) => {
            let value = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if value.is_empty() {
                "default: true".to_string()
            } else {
                value
            }
        }
        Err(_) => "not set".to_string(),
    }
}

/// Get the current core.autocrlf setting value
pub fn get_autocrlf_value() -> String {
    let output = Command::new("git")
        .args(&["config", "--get", "core.autocrlf"])
        .output();
    
    match output {
        Ok(out) => {
            let value = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if value.is_empty() {
                "default: false".to_string()
            } else {
                value
            }
        }
        Err(_) => "not set".to_string(),
    }
}

/// Check if Git LFS is installed and return its status
pub fn is_git_lfs_installed() -> bool {
    Command::new("git")
        .args(&["lfs", "version"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
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
// 3. ✅ Check Git version (minimum required version)
// 4. Check for proper SSH key configuration
// 5. Check for .gitignore patterns that might cause issues
// 6. Check for very long filenames (macOS has limits)
// 7. ✅ Check for case sensitivity issues (macOS is case-insensitive by default)
// 8. ✅ Check for proper line ending configuration (core.autocrlf)
// 9. Check for Git credential helper configuration
// 10. ✅ Check for NFD/NFC normalization conflicts in existing repos
// 11. ✅ Check for case-duplicate filenames (identical names except for letter case)
// 12. ✅ Check for large files not tracked by LFS
