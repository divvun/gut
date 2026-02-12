---
icon: lucide/terminal
---

# Command Overview

gut provides a comprehensive set of commands for managing multiple GitHub repositories. This page provides an overview of all available commands.

## Repository Operations

### clone
Clone all repositories matching a pattern. Automatically detects and pulls Git LFS objects for repos that use LFS.

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
Pull the current branch of all local repositories that match a pattern. Automatically pulls Git LFS objects for repos that use LFS.

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

### rename team
Rename a GitHub organisation team. The new slug is auto-generated by GitHub from the new name.

```bash
gut rename team <team-slug> <new-name> -o <org>
```

!!! warning "Team References"
    Renaming a team may invalidate references in issues and discussions. Requires typing "YES" to confirm.

## Labels

Manage GitHub labels in bulk across repositories.

### label list
List all labels for repositories matching a pattern.

```bash
gut label list -o <org> -r "<regex>"
```

### label create
Create a label across all matching repositories.

```bash
gut label create -o <org> -r "<regex>" \
  --name "bug" --color "d73a4a" \
  --description "Something isn't working"
```

### label delete
Delete a label from all matching repositories.

```bash
gut label delete -o <org> -r "<regex>" --name "bug"
```

### label rename
Rename a label (and optionally update color/description) across all matching repositories.

```bash
gut label rename -o <org> -r "<regex>" \
  --name "bug" --new-name "defect" \
  --color "ff0000"
```

## Health

### health
Comprehensive health check for repositories and system configuration.

```bash
# Check repos for a single owner
gut health -o <org>

# Check all owners
gut health --all-owners
```

**Checks performed:**

- **NFD/NFC normalization**: Filenames with decomposed Unicode that cause conflicts on macOS
- **Case duplicates**: Files differing only in case (e.g., `File.txt` vs `file.txt`)
- **Large files**: Files exceeding threshold that should use Git LFS (default: 50 MB)
- **Long paths**: Filenames/paths exceeding Windows compatibility limits
- **System config**: Git version, `core.precomposeUnicode`, `core.autocrlf`, Git LFS installation

**Options:**

- `--large-file-mb <SIZE>` — Size threshold in MB for LFS check (default: 50)
- `--filename-length-bytes <LENGTH>` — Filename length warning threshold (default: 200)
- `--path-length-bytes <LENGTH>` — Full path length warning threshold (default: 400)

## Topics

Topics are GitHub repository tags that help organize and discover repositories.

### topic list
List topics for all repositories that match a pattern.

```bash
gut topic list -o <org> -r "<regex>"

# JSON output
gut topic list -o <org> -r "<regex>" --json
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

### show repository
Show detailed access information for a specific repository, including teams and collaborators with permission levels and affiliation.

```bash
gut show repository <repo-name> -o <org>
```

**Output:**

- Teams table: Team Slug, Team Name, Permission
- Collaborators table: Username, Permission, Affiliation (org/direct/outside)

### show access
Show user access levels across repositories.

```bash
# Compact view (one column per user)
gut show access <username1> <username2> -o <org> -r "<regex>"

# Detailed view (one row per user/repo)
gut show access <username1> -o <org> --long
```

### show teams
Show all teams in an organisation.

```bash
# Flat table
gut show teams -o <org>

# Hierarchical tree showing parent/child relationships
gut show teams -o <org> --tree
```

### show team
Show details of a specific team including members, repositories, and parent/child teams.

```bash
gut show team <team-slug> -o <org>
```

### show members
Show all members in an organization with their roles and 2FA status.

```bash
gut show members -o <org>
```

### set owner
Set the default owner (organisation or user) for all commands.

```bash
gut set owner <owner-name>
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
