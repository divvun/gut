---
icon: lucide/terminal
---

# Command Overview

gut provides a comprehensive set of commands for managing multiple GitHub repositories. This page provides an overview of all available commands.

## Repository Operations

### clone
Clone all repositories matching a pattern.

```bash
gut clone -o <org> -r "<regex>"
```

**Example:**
```bash
gut clone -o myorg -r "lang-.*"
```

### fetch
Fetch all local repositories that match a pattern.

```bash
gut fetch -o <org> -r "<regex>"
```

### pull
Pull the current branch of all local repositories that match a pattern.

```bash
gut pull -o <org> -r "<regex>"
```

### push
Push the provided branch to remote server for all repositories that match a pattern.

```bash
gut push -o <org> -r "<regex>" --branch <branch>
```

### status
Show git status of all repositories that match a pattern.

```bash
gut status -o <org> -r "<regex>" --verbose
```

**Output columns:**

- **Repo Count**: Number of matched repos
- **Dirty**: Number of repos with modifications
- **fetch/push**: Number of repos needing fetch/push relative to remote
- **U**: Number of untracked files
- **D**: Number of deleted files
- **M**: Number of modified files
- **C**: Number of conflicted files
- **A**: Number of added files

## Git Operations

### checkout
Checkout a branch in all repositories that match a pattern or topic.

```bash
gut checkout -o <org> -r "<regex>" --branch <branch>
```

### commit
Add all changes and commit with the provided message for all local repositories.

```bash
gut commit -o <org> -r "<regex>" --message "commit message"
```

**Note:** This command will abort if there are conflicts or no changes.

### merge
Merge a branch into the current branch for all repositories that match a pattern.

```bash
gut merge -o <org> -r "<regex>" --branch <branch> --abort-if-conflict
```

Uses fast-forward strategy when possible. If conflicts occur, you must resolve them manually or use `--abort-if-conflict`.

### clean
Simulate `git clean -f -d` for all local repositories that match a pattern.

```bash
gut clean -o <org> -r "<regex>"
```

## Repository Management

### make
Change repository visibility (public/private).

```bash
# Make repositories private
gut make -o <org> -r "<regex>" private

# Make repositories public (requires confirmation)
gut make -o <org> -r "<regex>" public
```

!!! warning "Public Repositories"
    Making repositories public requires explicit confirmation by entering 'YES' at the prompt.

### set info
Set description and/or website for all repositories that match a pattern.

```bash
gut set info -o <org> -r "<regex>" \
  --description "Repository description" \
  --website "https://example.com"
```

You can also use scripts to generate descriptions dynamically:

```bash
gut set info -o <org> -r "<regex>" \
  --des-script ./generate-desc.sh \
  --web-script ./generate-url.sh
```

The script receives repository name as `$1` and organization name as `$2`.

### branch
Manage default and protected branches.

```bash
# Set default branch
gut branch default -o <org> -r "<regex>" --branch main

# Set protected branch
gut branch protect -o <org> -r "<regex>" --branch main

# Unprotect branch
gut branch unprotect -o <org> -r "<regex>" --branch develop
```

### create
Create resources (repos, branches, teams, discussions).

```bash
# Create a branch in matching repos
gut create branch -o <org> -r "<regex>" --branch feature/new

# Create a new team
gut create team -o <org> --name "Backend Team"

# Create a repository
gut create repo -o <org> --name "new-repo"
```

### transfer
Transfer repositories to another organization.

```bash
gut transfer -o <source-org> -r "<regex>" --target <target-org>
```

## Team & User Management

### add users
Invite users to an organization by username.

```bash
gut add users -o <org> \
  -u user1 user2 user3 \
  --role member
```

**Roles:** `member`, `admin` (organization) or `member`, `maintainer` (team)

Optionally invite to a team:

```bash
gut add users -o <org> -t team-slug \
  -u user1 user2 \
  --role member
```

### invite users
Invite users to an organization by email.

```bash
gut invite users -o <org> \
  -e user1@example.com user2@example.com \
  --role member
```

**Roles:** `member`, `admin`, `billing_manager`

### add repos
Add repositories to a team.

```bash
gut add repos -o <org> -t team-slug -r "<regex>"
```

### remove users
Remove users from an organization.

```bash
gut remove users -o <org> -u user1 user2
```

### remove repos
Remove repositories from a team.

```bash
gut remove repositories -o <org> -t team-slug -r "<regex>"
```

### set permission
Set access permissions for a team.

```bash
gut set permission -o <org> -t team-slug \
  -r "<regex>" \
  --permission push
```

**Permissions:** `pull`, `push`, `admin`, `maintain`, `triage`

## Topics

Topics are GitHub repository tags that help organize and discover repositories.

### topic get
Get topics for all repositories that match a pattern.

```bash
gut topic get -o <org> -r "<regex>"
```

### topic set
Set (replace) topics for all repositories.

```bash
gut topic set -o <org> -r "<regex>" \
  -t topic1 topic2 topic3
```

### topic add
Add topics to existing topics.

```bash
gut topic add -o <org> -r "<regex>" \
  -t new-topic
```

### topic apply
Apply a script to all repositories that have specific topics.

```bash
# Apply to repos with a specific topic
gut topic apply -o <org> -t backend -s ./script.sh

# Apply to repos with topics matching a pattern
gut topic apply -o <org> -r "service-.*" -s ./script.sh
```

## Hooks & Secrets

### hook create
Create webhooks for all repositories matching a pattern.

```bash
gut hook create -o <org> -r "<regex>" \
  --url "https://example.com/webhook" \
  --method json \
  --events push pull_request
```

**Methods:** `json`, `form`

**Common events:** `push`, `pull_request`, `issues`, `release`, `workflow_run`

### hook delete
Delete all webhooks for repositories matching a pattern.

```bash
gut hook delete -o <org> -r "<regex>"
```

### set secret
Set a secret for all repositories that match a pattern.

```bash
gut set secret -o <org> -r "<regex>" \
  --name SECRET_NAME \
  --value "secret-value"
```

!!! warning "Secrets"
    Be careful when setting secrets. Ensure you're using the correct repositories and never commit secrets to version control.

## Script Execution

### apply
Apply a custom script to all local repositories that match a pattern.

```bash
gut apply -o <org> -r "<regex>" -s ./my-script.sh
```

This is gut's most powerful command - it allows you to run **any** custom operation across multiple repositories. The script is executed in each repository's directory.

**Example script:**
```bash
#!/bin/bash
# Update dependencies
cargo update
git add Cargo.lock
git commit -m "Update dependencies"
```

## CI/CD

### ci generate
Generate CI configuration for repositories.

```bash
gut ci generate -o <org> -r "<regex>"
```

### ci export
Export data file for CI generation.

```bash
gut ci export -o <org> -r "<regex>"
```

## Workflows

### workflow run
Run or rerun workflows.

```bash
# Rerun the most recent workflow
gut workflow run -o <org> -r "<regex>"

# Send repository_dispatch event
gut workflow run -o <org> -r "<regex>" --event-type deploy
```

## Configuration

### show config
Display your current gut configuration.

```bash
gut show config
```

### show repositories
List all repositories matching a pattern.

```bash
gut show repositories -o <org> -r "<regex>"
```

### show users
Show all users in an organization.

```bash
gut show users -o <org>
```

### set organisation
Set the default organization for all commands.

```bash
gut set organisation <org-name>
```

## Template System

The template system allows you to apply consistent changes across repositories using Git patch sets.

### template generate
Generate a new project from a template.

```bash
gut template generate -o <org> --template <template-name> --name <new-repo>
```

### template apply
Apply template changes to repositories.

```bash
gut template apply -o <org> -r "<regex>" --template <template-path>
```

## Getting Help

For any command, use the `--help` flag:

```bash
gut --help
gut <command> --help
gut <command> <subcommand> --help
```
