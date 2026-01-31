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
        eprintln!(
            "\n{} {}",
            "⚠️  Warning:".yellow().bold(),
            self.title.yellow()
        );
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

/// Minimum required Git version
const MIN_GIT_VERSION: (u32, u32, u32) = (1, 7, 10);

/// Get the current Git version as a string (e.g., "2.39.3")
pub fn get_git_version() -> Option<String> {
    let output = Command::new("git").args(["--version"]).output().ok()?;
    let version_output = String::from_utf8_lossy(&output.stdout);
    version_output
        .split_whitespace()
        .nth(2)
        .map(|s| s.to_string())
}

/// Get a git config value, returning None if not set or on error
fn get_git_config(key: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--get", key])
        .output()
        .ok()?;

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

/// Check if Git LFS is installed
fn is_git_lfs_installed() -> bool {
    Command::new("git")
        .args(["lfs", "version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Parse a Git version string into (major, minor, patch) tuple.
/// Returns None if parsing fails. Missing patch version defaults to 0.
fn parse_git_version(version_str: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version_str.split('.').collect();
    if parts.len() < 2 {
        return None;
    }

    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts
        .get(2)
        .and_then(|p| p.parse::<u32>().ok())
        .unwrap_or(0);

    Some((major, minor, patch))
}

// ─── Check functions (return warnings if issues found) ───

/// Check if Git version meets minimum requirements
fn check_git_version() -> Option<HealthWarning> {
    let version_str = get_git_version()?;
    let (major, minor, patch) = parse_git_version(&version_str)?;

    if (major, minor, patch) < MIN_GIT_VERSION {
        let installed_version = if patch == 0 {
            format!("{}.{}", major, minor)
        } else {
            format!("{}.{}.{}", major, minor, patch)
        };

        return Some(HealthWarning {
            title: "Git version too old".to_string(),
            message: format!(
                "Git version {} is too old. Minimum required is {}.{}.{}.",
                installed_version, MIN_GIT_VERSION.0, MIN_GIT_VERSION.1, MIN_GIT_VERSION.2
            ),
            suggestion: Some(format!(
                "Update Git to version {}.{}.{} or newer:\n   \
                macOS: brew upgrade git\n   \
                Linux: Use your distribution's package manager to update git",
                MIN_GIT_VERSION.0, MIN_GIT_VERSION.1, MIN_GIT_VERSION.2
            )),
        });
    }

    None
}

/// Check if core.precomposeUnicode is properly set on macOS
///
/// Since Git 1.7.10, the default behavior on macOS is to use precomposed Unicode (NFC),
/// so it's OK if this setting is not explicitly set. We only warn if it's explicitly
/// set to false.
fn check_precompose_unicode() -> Option<HealthWarning> {
    let value = get_git_config("core.precomposeUnicode")?;

    // Only warn if explicitly set to false (empty/unset is OK as default is true since Git 1.7.10)
    if value.to_lowercase() == "false" {
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
    let value = get_git_config("core.autocrlf")?;

    // Warn if set to "true" on Unix systems (problematic)
    // "false", "input", or empty (unset) are all OK
    if value.to_lowercase() == "true" {
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
    if !is_git_lfs_installed() {
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

// ─── Public getters for display purposes ───

/// Get the current core.precomposeUnicode setting value for display
pub fn get_precompose_unicode_value() -> String {
    get_git_config("core.precomposeUnicode").unwrap_or_else(|| "default: true".to_string())
}

/// Get the current core.autocrlf setting value for display
pub fn get_autocrlf_value() -> String {
    get_git_config("core.autocrlf").unwrap_or_else(|| "default: false".to_string())
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
// 6. ✅ Check for very long filenames (macOS has limits)
// 7. ✅ Check for case sensitivity issues (macOS is case-insensitive by default)
// 8. ✅ Check for proper line ending configuration (core.autocrlf)
// 9. Check for Git credential helper configuration
// 10. ✅ Check for NFD/NFC normalization conflicts in existing repos
// 11. ✅ Check for case-duplicate filenames (identical names except for letter case)
// 12. ✅ Check for large files not tracked by LFS
