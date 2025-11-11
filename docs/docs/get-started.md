---
icon: lucide/rocket
---

# Get Started

This guide will help you install gut and get it configured for your GitHub organization.

## Installation

### Download Pre-built Binaries

Download the latest nightly build for your platform:

- [Linux](https://pahkat.uit.no/devtools/download/gut?channel=nightly&platform=linux) (x86_64)
- [macOS](https://pahkat.uit.no/devtools/download/gut?channel=nightly&platform=macos) (x86_64)
- [Windows](https://pahkat.uit.no/devtools/download/gut?channel=nightly&platform=windows) (i686)

Extract the archive and move the binary to somewhere on your `$PATH`.

### Build from Source

If you prefer to build from source:

1. Install [Rust](https://www.rust-lang.org/learn/get-started)
2. Clone the repository:
   ```bash
   git clone https://github.com/divvun/gut.git
   cd gut
   ```
3. Build and install:
   ```bash
   cargo install --path .
   ```

The `gut` binary will be installed to `~/.cargo/bin/`.

## Initial Setup

### 1. Create a GitHub Personal Access Token

gut needs a personal access token to interact with GitHub's API:

1. Go to [GitHub Settings > Personal Access Tokens](https://github.com/settings/tokens)
2. Click "Generate new token (classic)"
3. Give it a descriptive name (e.g., "gut CLI")
4. Select the required scopes:
   - `repo` (Full control of private repositories)
   - `admin:org` (Full control of orgs and teams) - if managing organizations
   - `delete_repo` (Delete repositories) - if you need this capability
5. Click "Generate token"
6. **Copy the token** - you won't be able to see it again!

### 2. Initialize gut Configuration

Run the initialization command:

```bash
gut init -r <root-dir> -t <token>
```

**Parameters:**

- `<root-dir>`: The directory where gut will store all cloned repositories
  - Repositories are organized as `<root-dir>/<organization>/<repo-name>`
  - Example: `/Users/you/projects/repos`

- `<token>`: The GitHub personal access token you created in step 1

**Example:**

```bash
gut init -r ~/gut-repos -t ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

This creates a configuration file in your system's config directory (e.g., `~/.config/gut/` on Linux/macOS).

### 3. Set a Default Organization (Optional)

To avoid specifying `-o <organization>` with every command, set a default:

```bash
gut set organisation <org-name>
```

For example:

```bash
gut set organisation divvun
```

## SSH Access (Optional)

By default, gut uses HTTPS for git operations. If you prefer SSH:

1. Set up an SSH key for GitHub following [these instructions](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/generating-a-new-ssh-key-and-adding-it-to-the-ssh-agent)
2. When running `gut init`, the tool will detect your SSH setup

!!! note "Protocol Selection"
    The choice between HTTPS and SSH is made during initialization. HTTPS is recommended for simplicity, while SSH is useful if you have complex authentication requirements or use hardware security keys.

## Verify Installation

Check that everything is working:

```bash
# View your configuration
gut show config

# List repositories in your organization
gut show repositories -o <org-name> -r ".*"
```

If these commands work, you're all set!

## Next Steps

Now that gut is installed and configured, you can:

- Explore [common commands](usage/common-commands.md) to see what gut can do
- Check out the [usage overview](usage/overview.md) for detailed command documentation
- Learn about the [architecture](architecture.md) if you want to contribute

## Getting Help

- Run `gut --help` to see all available commands
- Run `gut <command> --help` for help with a specific command
- Check the [GitHub issues](https://github.com/divvun/gut/issues) for known problems
- Review the [USAGE.md](https://github.com/divvun/gut/blob/main/USAGE.md) file for additional examples
