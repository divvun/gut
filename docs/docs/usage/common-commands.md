---
icon: lucide/zap
---

# Common Commands

This page covers the most frequently used gut commands with practical examples.

## Working with Repositories

### Cloning Multiple Repositories

Clone all repositories from your organization that match a pattern:

```bash
# Clone all language repositories
gut clone -o myorg -r "^lang-.*"

# Clone all backend services
gut clone -o myorg -r ".*-service$"

# Clone everything
gut clone -o myorg -r ".*"
```

!!! tip "First Time Setup"
    After initial configuration, cloning all your organization's repos is typically the first command you'll run.

### Checking Repository Status

See the status of all your local repositories:

```bash
# Basic status
gut status -o myorg -r ".*"

# Verbose status (shows detailed information)
gut status -o myorg -r ".*" --verbose

# Status for specific repo group
gut status -o myorg -r "^project-.*"
```

The status command shows you at a glance which repos are dirty, need fetching, or have uncommitted changes.

### Fetching and Pulling

Keep your local repositories up to date:

```bash
# Fetch all repos
gut fetch -o myorg -r ".*"

# Pull all repos on main branch
gut checkout -o myorg -r ".*" --branch main
gut pull -o myorg -r ".*"
```

## Bulk Operations

### Committing Across Repositories

Add and commit changes across multiple repositories:

```bash
# Commit changes in all repos matching pattern
gut commit -o myorg -r "lang-.*" \
  --message "Update copyright year to 2025"

# This is equivalent to running in each repo:
# git add -A
# git commit -m "Update copyright year to 2025"
```

### Applying Scripts

The `apply` command is gut's superpower - run any script across multiple repos:

```bash
# Update dependencies
gut apply -o myorg -r ".*-service$" -s update-deps.sh

# Run tests
gut apply -o myorg -r "^lib-.*" -s run-tests.sh

# Format code
gut apply -o myorg -r ".*" -s format.sh
```

**Example script** (`update-deps.sh`):
```bash
#!/bin/bash
set -e

# Update Rust dependencies
if [ -f "Cargo.toml" ]; then
    cargo update
    git add Cargo.lock
fi

# Update npm dependencies
if [ -f "package.json" ]; then
    npm update
    git add package-lock.json
fi

# Commit if there are changes
if ! git diff --cached --quiet; then
    git commit -m "Update dependencies"
fi
```

### Branch Operations

Create, switch, or merge branches across repositories:

```bash
# Create a new branch in all repos
gut create branch -o myorg -r ".*" --branch feature/security-update

# Checkout existing branch
gut checkout -o myorg -r ".*" --branch develop

# Merge a branch
gut checkout -o myorg -r ".*" --branch main
gut merge -o myorg -r ".*" --branch develop --abort-if-conflict
```

## GitHub Management

### Managing Repository Settings

Update repository information:

```bash
# Set description for all repos
gut set info -o myorg -r "^lang-.*" \
  --description "Language implementation for X"

# Set both description and website
gut set info -o myorg -r "myproject-.*" \
  --description "MyProject components" \
  --website "https://myproject.dev"
```

### Managing Topics

Topics help organize and discover repositories:

```bash
# Add a topic to repositories
gut topic add -o myorg -r ".*-service$" -t microservice backend

# Set topics (replaces existing)
gut topic set -o myorg -r "^lang-sme" -t language sami production

# List current topics
gut topic list -o myorg -r ".*"

# Apply script to repos with specific topic
gut topic apply -o myorg -t backend -s deploy.sh
```

### Setting Secrets

Add secrets to multiple repositories:

```bash
# Set deployment token for all services
gut set secret -o myorg -r ".*-service$" \
  --name DEPLOY_TOKEN \
  --value "ghp_xxxxxxxxxxxx"

# Set API key for backend services
gut set secret -o myorg -r "backend-.*" \
  --name API_KEY \
  --value "sk_live_xxxxx"
```

!!! warning
    Never commit secrets or include them in scripts stored in version control.

## Team Management

### Adding Team Members

Invite users to your organization:

```bash
# Invite by username
gut add users -o myorg \
  -u alice bob charlie \
  --role member

# Invite by email
gut invite users -o myorg \
  -e alice@example.com bob@example.com \
  --role member

# Add users to a specific team
gut add users -o myorg -t developers \
  -u alice bob \
  --role member
```

### Managing Repository Access

Give teams access to repositories:

```bash
# Add repos to team with push access
gut add repos -o myorg -t backend-team -r "backend-.*"
gut set permission -o myorg -t backend-team -r "backend-.*" --permission push

# Give admin access
gut set permission -o myorg -t core-team -r ".*" --permission admin
```

## Practical Workflows

### Starting a New Feature

```bash
# 1. Fetch latest changes
gut fetch -o myorg -r "myproject-.*"

# 2. Checkout main branch
gut checkout -o myorg -r "myproject-.*" --branch main

# 3. Pull latest
gut pull -o myorg -r "myproject-.*"

# 4. Create feature branch
gut create branch -o myorg -r "myproject-.*" --branch feature/new-api

# 5. Make your changes locally in each repo...

# 6. Commit across all repos
gut commit -o myorg -r "myproject-.*" --message "Add new API endpoints"

# 7. Push to remote
gut push -o myorg -r "myproject-.*" --branch feature/new-api
```

### Applying Security Updates

```bash
# Create a script to update dependencies and run tests
cat > security-update.sh << 'EOF'
#!/bin/bash
set -e

# Update dependencies
cargo update

# Run tests
cargo test

# Stage changes
git add Cargo.lock
EOF

chmod +x security-update.sh

# Apply to all Rust projects
gut apply -o myorg -r ".*" -s ./security-update.sh

# Check status
gut status -o myorg -r ".*"

# Commit if tests passed
gut commit -o myorg -r ".*" --message "Security: Update dependencies"

# Push
gut push -o myorg -r ".*" --branch main
```

### Updating Documentation

```bash
# Create script to update README
cat > update-readme.sh << 'EOF'
#!/bin/bash
# Add build badge to README if it doesn't exist
if ! grep -q "Build Status" README.md; then
    repo_name=$(basename $(pwd))
    org_name=$(basename $(dirname $(pwd)))
    echo "[![Build Status](https://ci.example.com/$org_name/$repo_name/badge.svg)](https://ci.example.com/$org_name/$repo_name)" >> README.md
    git add README.md
fi
EOF

chmod +x update-readme.sh

# Apply to all repos
gut apply -o myorg -r ".*" -s ./update-readme.sh
gut commit -o myorg -r ".*" --message "docs: Add build badge"
gut push -o myorg -r ".*"
```

## Tips and Tricks

### Using Default Organization

Avoid typing `-o <org>` every time:

```bash
gut set organisation myorg

# Now you can omit -o:
gut status -r ".*"
gut clone -r "new-project"
```

### Regex Patterns

Common regex patterns for filtering repos:

```bash
# All repos starting with "lang-"
-r "^lang-.*"

# All repos ending with "-service"
-r ".*-service$"

# All repos containing "backend"
-r ".*backend.*"

# Exact match
-r "^exact-name$"

# Everything
-r ".*"

# Multiple prefixes (requires regex alternation)
-r "^(frontend|backend|api)-.*"
```

### Dry Run Pattern

Before applying changes, always check what repos will be affected:

```bash
# See which repos match
gut show repositories -o myorg -r "^lang-.*"

# Check their status
gut status -o myorg -r "^lang-.*"

# Then apply your changes
gut apply -o myorg -r "^lang-.*" -s script.sh
```

### Combining with Standard Git

gut doesn't prevent you from using regular git commands. You can:

1. Use gut for bulk operations
2. Use git for fine-grained control in specific repos

```bash
# Use gut to checkout a branch everywhere
gut checkout -o myorg -r ".*" --branch develop

# Navigate to a specific repo and use git normally
cd ~/gut-repos/myorg/specific-repo
git log
git diff
git commit --amend
```
