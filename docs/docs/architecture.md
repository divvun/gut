---
icon: lucide/box
---

# Architecture

This document describes gut's internal architecture, module structure, and design patterns. This information is useful for contributors and those interested in understanding how gut works under the hood.

## Project Structure

gut is a Rust command-line application built with standard Rust ecosystem tools:

- **Build system**: Cargo
- **CLI framework**: [clap](https://docs.rs/clap) v4 with derive macros
- **Git operations**: [git2](https://docs.rs/git2) (libgit2 bindings)
- **GitHub API**: [reqwest](https://docs.rs/reqwest) for REST, [graphql_client](https://docs.rs/graphql_client) for GraphQL
- **Parallel execution**: [rayon](https://docs.rs/rayon)
- **Serialization**: [serde](https://docs.rs/serde)

## Module Organization

```
src/
├── main.rs              # Entry point, logging setup
├── cli.rs               # Command-line interface definitions
├── config.rs            # Configuration management
├── commands/            # Command implementations
│   ├── mod.rs
│   ├── clone.rs
│   ├── apply.rs
│   ├── status.rs
│   └── ...             # One file per command
├── git/                 # Git operations wrapper
│   ├── mod.rs
│   ├── clone.rs
│   ├── commit.rs
│   ├── branch.rs
│   └── ...             # Git operation modules
├── github/              # GitHub API interactions
│   ├── mod.rs
│   ├── rest.rs          # REST API calls
│   ├── graphql.rs       # GraphQL queries
│   └── models.rs        # Data structures
├── filter.rs            # Repository filtering logic
├── user.rs              # GitHub user/token management
├── convert.rs           # Type conversions
├── path.rs              # Path utilities
└── toml.rs              # TOML file operations
```

## Core Modules

### CLI Module (`cli.rs`)

Defines all commands and their arguments using clap's derive macros:

```rust
#[derive(Debug, Parser)]
#[command(name = "gut", about = "git multirepo maintenance tool")]
pub struct Args {
    #[arg(long, value_enum, default_value = "table")]
    pub format: Option<OutputFormat>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Clone(CloneArgs),
    Apply(ApplyArgs),
    Status(StatusArgs),
    // ... more commands
}
```

Each command variant contains a struct with its specific arguments. Commands also have convenient aliases (e.g., `co` for `checkout`, `ap` for `apply`).

### Commands Module (`commands/`)

Each command is implemented in its own module with a `run()` method:

```rust
impl CloneArgs {
    pub fn run(&self, common_args: &Args) -> Result<()> {
        // Command implementation
    }
}
```

The command pattern keeps related functionality together and makes the codebase easy to navigate.

**Key command modules:**

- **Repository operations**: `clone`, `fetch`, `pull`, `push`, `status`
- **Git operations**: `checkout`, `commit`, `merge`, `branch_*`
- **GitHub management**: `create_*`, `set_*`, `add_*`, `remove_*`
- **Topics**: `topic_add`, `topic_set`, `topic_get`, `topic_apply`
- **Utilities**: `apply`, `template/*`, `workflow_*`

### Git Module (`git/`)

Wraps libgit2 operations to provide a higher-level, gut-specific API:

```rust
pub trait Clonable {
    fn clone(&self, path: &Path) -> Result<Repository>;
}

pub fn commit_all(repo: &Repository, message: &str) -> Result<()>;
pub fn checkout_branch(repo: &Repository, branch: &str) -> Result<()>;
pub fn merge_branch(repo: &Repository, branch: &str, ff_only: bool) -> Result<()>;
// ... more operations
```

All git operations should use these wrappers rather than shelling out to git commands. This provides:

- Consistent error handling
- Better performance (no process spawning)
- Cross-platform compatibility
- Type safety

### GitHub Module (`github/`)

Handles all GitHub API interactions through two sub-modules:

#### REST API (`github/rest.rs`)

Used for mutations and operations not efficiently done via GraphQL:

```rust
// HTTP helper functions
fn get(url: &str, token: &str, accept: Option<&str>) -> Result<Response>;
fn post<T>(url: &str, body: &T, token: &str) -> Result<Response>;
fn patch<T>(url: &str, body: &T, token: &str) -> Result<Response>;
fn delete(url: &str, token: &str) -> Result<Response>;

// Repository operations
pub fn update_default_branch(org: &str, repo: &str, branch: &str, token: &str) -> Result<()>;
pub fn set_repo_visibility(org: &str, repo: &str, private: bool, token: &str) -> Result<()>;
pub fn create_webhook(org: &str, repo: &str, config: &WebhookConfig, token: &str) -> Result<()>;
// ... more operations
```

#### GraphQL API (`github/graphql.rs`)

Used for efficient bulk queries:

```rust
// Define queries using graphql_client
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "github.graphql",
    query_path = "user_query.graphql",
    response_derives = "Debug"
)]
struct OrganizationRepositories;

pub fn get_org_repos(org: &str, token: &str) -> Result<Vec<RemoteRepo>>;
pub fn get_org_members(org: &str, token: &str) -> Result<Vec<OrgMember>>;
```

GraphQL is preferred for fetching data because:
- Single request can fetch all needed data
- Reduced API rate limit consumption
- Faster than multiple REST calls

Both modules use bearer token authentication and include the custom User-Agent header.

### Configuration (`config.rs`)

Manages gut's configuration stored in the user's config directory:

```rust
pub struct Config {
    pub root: String,              // Root directory for repos
    pub default_org: Option<String>, // Default organization
    pub use_https: bool,           // Protocol preference
}
```

Configuration location:
- Linux: `~/.config/gut/`
- macOS: `~/Library/Application Support/gut/`
- Windows: `%APPDATA%\gut\`

The GitHub token is stored separately in the same directory.

### Filtering (`filter.rs`)

Provides repository filtering by regex patterns:

```rust
pub fn filter_repos(repos: &[RemoteRepo], pattern: &str) -> Result<Vec<RemoteRepo>> {
    let re = Regex::new(pattern)?;
    Ok(repos.iter()
        .filter(|r| re.is_match(&r.name))
        .cloned()
        .collect())
}
```

This is a core feature that enables bulk operations on repository subsets.

## Design Patterns

### 1. Command Pattern

Each subcommand is a struct that implements a `run()` method:

```rust
pub struct ApplyArgs {
    #[arg(short, long)]
    organisation: String,
    #[arg(short, long)]
    regex: String,
    #[arg(short, long)]
    script: PathBuf,
}

impl ApplyArgs {
    pub fn run(&self, common_args: &Args) -> Result<()> {
        // Implementation
    }
}
```

Benefits:
- Clear separation of concerns
- Easy to test individual commands
- Simple to add new commands

### 2. Repository Root Structure

All repositories are organized consistently:

```
<root>/
  ├── <organization-1>/
  │   ├── repo-1/
  │   ├── repo-2/
  │   └── repo-3/
  └── <organization-2>/
      ├── repo-4/
      └── repo-5/
```

This predictable structure allows:
- Easy discovery of cloned repos
- Conflict-free organization of multiple orgs
- Simple path construction

### 3. Parallel Execution

Operations across multiple repositories use rayon for parallelization:

```rust
use rayon::prelude::*;

repos.par_iter()
    .for_each(|repo| {
        // Operation on each repo
    });
```

This dramatically speeds up bulk operations like `fetch`, `status`, and `apply`.

### 4. Dual API Strategy

gut uses both GitHub's REST and GraphQL APIs strategically:

**GraphQL for:**
- Fetching repository lists
- Getting organization members
- Bulk data retrieval

**REST for:**
- Creating/updating resources
- Setting secrets (encryption required)
- Operations not available in GraphQL

### 5. Error Handling

Uses `anyhow` for ergonomic error handling with context:

```rust
use anyhow::{Context, Result};

fn do_something() -> Result<()> {
    let config = Config::from_file()
        .context("Failed to load configuration")?;
    // ...
    Ok(())
}
```

Custom error types are defined using `thiserror` for domain-specific errors.

## Data Flow

### Typical Command Execution

1. **CLI Parsing**: clap parses command-line arguments into structured data
2. **Config Loading**: Configuration loaded from user's config directory
3. **API Query**: GitHub API (GraphQL/REST) fetches repository data
4. **Filtering**: Regex pattern filters repositories
5. **Parallel Execution**: Operation applied to each matching repo using rayon
6. **Output**: Results formatted as table or JSON

### Example: `gut status` Flow

```
User runs: gut status -o myorg -r "lang-.*"
                ↓
         Parse CLI args
                ↓
       Load configuration
                ↓
    Fetch repos via GraphQL
                ↓
    Filter repos by regex
                ↓
  For each repo (in parallel):
    - Check if cloned locally
    - Run git status
    - Collect statistics
                ↓
    Aggregate results
                ↓
   Format as table/JSON
                ↓
    Display to user
```

## Testing

gut uses standard Rust testing:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocaptured
```

Tests use:
- **Unit tests**: In the same file as the code (`#[cfg(test)]` modules)
- **Integration tests**: In `tests/` directory
- **Mocking**: [proptest](https://docs.rs/proptest) for property-based testing
- **Temp files**: [tempfile](https://docs.rs/tempfile) for filesystem tests

## Development Guidelines

### Adding a New Command

1. Define command args struct in `src/commands/<name>.rs`:
   ```rust
   #[derive(Debug, Args)]
   pub struct MyCommandArgs {
       #[arg(short, long)]
       pub option: String,
   }
   ```

2. Implement the `run()` method:
   ```rust
   impl MyCommandArgs {
       pub fn run(&self, common_args: &Args) -> Result<()> {
           // Implementation
       }
   }
   ```

3. Add variant to `Commands` enum in `src/cli.rs`:
   ```rust
   #[derive(Debug, Subcommand)]
   pub enum Commands {
       // ... existing commands
       #[command(name = "mycommand")]
       MyCommand(MyCommandArgs),
   }
   ```

4. Add match arm in `src/main.rs`:
   ```rust
   match &common_args.command {
       // ... existing matches
       Commands::MyCommand(args) => args.run(&common_args),
   }
   ```

5. Export from `src/commands/mod.rs`:
   ```rust
   pub mod my_command;
   pub use my_command::*;
   ```

### Working with GitHub API

When adding new GitHub API functionality:

1. Check if it's available in GraphQL (preferred for queries)
2. Define the query in `user_query.graphql` if using GraphQL
3. Add the operation in `github/rest.rs` or `github/graphql.rs`
4. Include proper error handling and logging
5. Test with actual API calls (requires token)

### Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Run clippy lints (`cargo clippy`)
- Write doc comments for public APIs
- Keep functions focused and single-purpose
- Use descriptive variable names
- Add logging with `log::debug!()` for debugging

## Performance Considerations

- **Parallel operations**: Use rayon for independent repo operations
- **API batching**: GraphQL reduces API calls vs. REST
- **Rate limiting**: Be aware of GitHub API rate limits (5000/hour authenticated)
- **Git operations**: libgit2 is faster than shelling out to git
- **Incremental operations**: Only process repos that match filters

## Security

- Tokens stored in OS-specific secure locations
- Never log tokens or secrets
- Encrypt secrets before sending to GitHub API
- Validate user input, especially regex patterns
- Use HTTPS by default for cloning

## Future Architecture Considerations

Potential improvements:

- **Plugin system**: Allow external commands
- **Configuration profiles**: Multiple token/org combinations
- **Caching**: Cache repo lists to reduce API calls
- **Progress indicators**: Better feedback for long operations
- **Async operations**: Consider tokio for truly async operations
