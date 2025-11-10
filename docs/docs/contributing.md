---
icon: lucide/git-pull-request
---

# Contributing

Thank you for your interest in contributing to gut! This guide will help you get started with development.

## Getting Started

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs)
- **Git**: For version control
- **GitHub Account**: For testing GitHub API features

### Setting Up Your Development Environment

1. **Fork and clone the repository:**
   ```bash
   git clone https://github.com/YOUR-USERNAME/gut.git
   cd gut
   ```

2. **Build the project:**
   ```bash
   cargo build
   ```

3. **Run tests:**
   ```bash
   cargo test
   ```

4. **Run gut locally:**
   ```bash
   cargo run -- --help
   ```

## Development Workflow

### Building and Running

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (optimized)
cargo build --release

# Run directly
cargo run -- <command> <args>

# Install locally for testing
cargo install --path .
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocaptured

# Run tests for specific module
cargo test --package gut --lib commands::status
```

### Code Quality

Before submitting a pull request, ensure your code passes all checks:

```bash
# Format code
cargo fmt

# Check formatting (CI will run this)
cargo fmt -- --check

# Run clippy lints
cargo clippy

# Run clippy with strict lints
cargo clippy -- -D warnings
```

### Generating Documentation

Generate and view the code documentation:

```bash
# Generate documentation
cargo doc

# Generate and open in browser
cargo doc --open

# Documentation will be in target/doc/gut/
```

!!! warning "Never call rustc directly"
    Always use `cargo` commands. Never call `rustc` directly to compile code.

## Development Guidelines

### Code Examples

Create examples in the `examples/` directory:

```bash
# Create an example
cat > examples/my_example.rs << 'EOF'
fn main() {
    println!("Example code");
}
EOF

# Run the example
cargo run --example my_example
```

### Project Structure

When adding new functionality:

- **Commands**: Add to `src/commands/<name>.rs`
- **Git operations**: Add to `src/git/<operation>.rs`
- **GitHub API**: Add to `src/github/rest.rs` or `src/github/graphql.rs`
- **Utilities**: Add to appropriate module or create new one

See the [Architecture](architecture.md) guide for detailed module structure.

### Adding a New Command

Follow these steps to add a new command:

1. **Create the command module:**

   Create `src/commands/my_command.rs`:
   ```rust
   use crate::cli::Args;
   use anyhow::Result;
   use clap::Args as ClapArgs;

   #[derive(Debug, ClapArgs)]
   pub struct MyCommandArgs {
       /// Organisation name
       #[arg(short, long)]
       pub organisation: String,

       /// Regex pattern to filter repositories
       #[arg(short, long)]
       pub regex: String,
   }

   impl MyCommandArgs {
       pub fn run(&self, _common_args: &Args) -> Result<()> {
           log::info!("Running my command");
           // Implementation here
           Ok(())
       }
   }
   ```

2. **Export from commands module:**

   Add to `src/commands/mod.rs`:
   ```rust
   pub mod my_command;
   pub use my_command::*;
   ```

3. **Add to CLI:**

   Add to `src/cli.rs`:
   ```rust
   #[derive(Debug, Subcommand)]
   pub enum Commands {
       // ... existing commands
       #[command(name = "mycommand", aliases = &["mc"])]
       MyCommand(MyCommandArgs),
   }
   ```

4. **Add to main dispatcher:**

   Add to `src/main.rs`:
   ```rust
   match &common_args.command {
       // ... existing matches
       Commands::MyCommand(args) => args.run(&common_args),
   }
   ```

5. **Test your command:**
   ```bash
   cargo run -- mycommand --help
   ```

### Working with Git

Use the git module wrappers, not shell commands:

```rust
// Good
use crate::git::{open_repo, commit_all, checkout_branch};

let repo = open_repo(&path)?;
checkout_branch(&repo, "main")?;
commit_all(&repo, "Update files")?;

// Bad - don't do this
use std::process::Command;
Command::new("git").args(&["commit", "-m", "message"]).output()?;
```

### Working with GitHub API

#### REST API

Add functions to `src/github/rest.rs`:

```rust
pub fn my_api_call(org: &str, repo: &str, token: &str) -> Result<ResponseType> {
    let url = format!("https://api.github.com/repos/{}/{}/endpoint", org, repo);
    let response = get(&url, token, None)?;

    if !response.status().is_success() {
        anyhow::bail!("API call failed: {}", response.status());
    }

    let data: ResponseType = response.json()?;
    Ok(data)
}
```

#### GraphQL API

1. Add query to `user_query.graphql`:
   ```graphql
   query MyQuery($org: String!) {
     organization(login: $org) {
       # fields
     }
   }
   ```

2. Define the query in `src/github/graphql.rs`:
   ```rust
   #[derive(GraphQLQuery)]
   #[graphql(
       schema_path = "github.graphql",
       query_path = "user_query.graphql",
       response_derives = "Debug"
   )]
   struct MyQuery;
   ```

3. Implement the function:
   ```rust
   pub fn my_graphql_query(org: &str, token: &str) -> Result<Vec<Data>> {
       let q = MyQuery::build_query(my_query::Variables {
           org: org.to_string(),
       });

       let res = query(token, &q)?;
       let response_body: Response<my_query::ResponseData> = res.json()?;

       // Process response...
       Ok(data)
   }
   ```

### Error Handling

Use `anyhow` for error handling with context:

```rust
use anyhow::{Context, Result};

fn my_function() -> Result<()> {
    let config = Config::from_file()
        .context("Failed to load configuration")?;

    let repos = fetch_repos(&config.organization)
        .with_context(|| format!("Failed to fetch repos for {}", config.organization))?;

    Ok(())
}
```

### Logging

Use the `log` crate for debugging:

```rust
log::debug!("Processing repository: {}", repo.name);
log::info!("Cloning {} repositories", count);
log::warn!("Repository {} has uncommitted changes", repo.name);
log::error!("Failed to access repository: {}", error);
```

Run with logging enabled:
```bash
RUST_LOG=gut=debug cargo run -- <command>
```

## Testing Your Changes

### Manual Testing

1. **Install your development version:**
   ```bash
   cargo install --path .
   ```

2. **Test with a test organization:**
   - Create a test GitHub organization
   - Clone some test repos
   - Run commands against test repos

3. **Test edge cases:**
   - Empty repositories
   - Repositories with conflicts
   - Invalid regex patterns
   - Network failures

### Automated Testing

Add unit tests in the same file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        let result = my_function();
        assert!(result.is_ok());
    }
}
```

Integration tests go in the `tests/` directory.

## Submitting Changes

### Commit Guidelines

- Write clear, descriptive commit messages
- Use conventional commit format when appropriate:
  - `feat:` for new features
  - `fix:` for bug fixes
  - `docs:` for documentation
  - `refactor:` for code refactoring
  - `test:` for adding tests

**Example:**
```
feat: add support for GitHub Actions workflow triggers

Add new `workflow run` command that allows triggering workflows
across multiple repositories using repository_dispatch events.

Fixes #123
```

### Pull Request Process

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make your changes and commit:**
   ```bash
   git add .
   git commit -m "feat: add awesome feature"
   ```

3. **Push to your fork:**
   ```bash
   git push origin feature/my-feature
   ```

4. **Open a Pull Request:**
   - Go to the gut repository on GitHub
   - Click "New Pull Request"
   - Select your fork and branch
   - Fill in the PR template
   - Request review

5. **Address review feedback:**
   - Make requested changes
   - Push new commits to the same branch
   - PR will update automatically

### PR Checklist

Before submitting, ensure:

- [ ] Code compiles without errors
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Clippy lints pass (`cargo clippy`)
- [ ] Documentation is updated if needed
- [ ] New functionality has tests
- [ ] Commit messages are clear

## Getting Help

- **Questions**: Open a [GitHub Discussion](https://github.com/divvun/gut/discussions)
- **Bugs**: Open a [GitHub Issue](https://github.com/divvun/gut/issues)
- **Architecture**: Read the [Architecture Guide](architecture.md)
- **Rust Help**: Check [The Rust Book](https://doc.rust-lang.org/book/)

## Code of Conduct

Be respectful, constructive, and welcoming to all contributors. We want gut to be a friendly and inclusive project.

## License

By contributing to gut, you agree that your contributions will be licensed under the same license as the project.
