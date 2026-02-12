# gut

Pronounced `/gʉːt/`, as in the Norwegian word for *boy*.

[![](https://divvun-tc.thetc.se/api/github/v1/repository/divvun/gut/main/badge.svg)](https://divvun-tc.thetc.se/api/github/v1/repository/divvun/gut/main/latest)

This is a Git(Hub) multirepo maintenance tool, designed specifically for Divvun. But it should be quite useful to others needing to maintain tens (or hundreds) of similarly structured GitHub repositories.

Using the `gut apply -s <script>` command, one can in practice run any git command on all repos (or a suitable subset, regex-selected on the repo names), not only the commands directly provided by `gut`.

We think it's pretty cool.

## Documentation

**📚 [Full Documentation](docs/)** - Complete guides and references

Quick links:

- [Get Started](docs/docs/get-started.md) - Installation and setup
- [Common Commands](docs/docs/usage/common-commands.md) - Most frequently used commands
- [Command Overview](docs/docs/usage/overview.md) - All available commands
- [Architecture](docs/docs/architecture.md) - Technical details
- [Contributing](docs/docs/contributing.md) - Development guide
- [Changelog](CHANGELOG.md) - Release history

### Viewing Documentation Locally

The documentation is built with [Zensical](https://zensical.org/). To view it locally:

```bash
# Install Zensical
pip install zensical

# Serve documentation
cd docs
zensical serve
```

Then open <http://localhost:8000> in your browser.

## Installation

### Pre-built binaries

Download from the [latest release page](https://github.com/divvun/gut/releases/latest).
Extract the archive, and move the binary to somewhere on your `$PATH`.

> ⚠️ Please note that all binaries are completely stand-alone, no external dependencies, not even the C library on Linux.

### Building from source

1. get [Rust](https://www.rust-lang.org/learn/get-started)
1. clone this repo: `git clone https://github.com/divvun/gut.git`
1. `cd gut`
1. `cargo install --path .`

If you get compilation errors related to SSL (esp. on the mac), try this variant for step 4. above:

`OPENSSL_NO_VENDOR=1 cargo install --path .`

## Setup

1. make a [personal access token](https://github.com/settings/tokens) in GitHub - allow everything. Make sure to store it in a safe place - the token replaces your username and password when accessing GitHub via `gut`.
1. run `gut init -r <root-dir> -t <token>`

`<token>` is the token created in step 1. above.
The `<root-dir>` is the directory where you want to store all repos processed by `gut`.
Below the `<root-dir>` dir, there will be one directory for every organisation you interact with, and within the organisation directory all repos are stored.

### SSH access over the `git` protocoll

To use the `git`/`ssh` protocol, you need to set up an `ssh` key for GitHub. Follow [these instructions](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/generating-a-new-ssh-key-and-adding-it-to-the-ssh-agent).

## Features

- **Multi-repo operations**: Clone, pull, push, commit, fetch across hundreds of repos with a single command
- **Regex filtering**: Target specific (subset of) repos using regex patterns (e.g., `-r "^lang-.*"`)
- **Topic filtering**: Filter repos by GitHub topics (e.g., `--topic lang`)
- **Multi-owner support**: Run commands across all owners (organizational and personal accounts) with `--all-owners` / `-a` flag
- **Progress bars**: Visual feedback for long-running operations
- **Parallel execution**: Network operations run in parallel for speed
- **Template system**: Apply consistent changes across repos using templates with placeholder substitution

## Usage

> **NB!** Please note that this is a potentially very powerful tool. Some commands require *organisation owner permissions*, and the most dangerous ones will require an *explicit confirmation*. If you get an error that the operation is not permitted, you probably do not have sufficient access to the repos involved.

There are some [usage instructions](USAGE.md) under development.

Then there are some use cases with example commands
[here](https://giellalt.github.io/infra/infraremake/HowToMergeUpdatesFromCore.html).

### Common Examples

```bash
# Clone all repos matching a pattern
gut clone -o myorg -r "^api-.*"

# Pull all repos across all organizations
gut pull --all-owners

# Fetch and check status of all repos matching regex
gut status -r ".*" --fetch

# Commit changes to all matching repos
gut commit -r "^lib-.*" -m "Update dependencies"

# Apply a script to multiple repos
gut apply -r ".*" -s ./my-script.sh
```

For more details on any command, use `gut <command> --help`.

There is also some rudimentary help text. Run `gut --help` to get an overview:

```
$ gut --help
git multirepo maintenance tool

Usage: gut [OPTIONS] <COMMAND>

Commands:
  add       Add users (to an organisation or team) or repositories (to a team)
  apply     Apply a script to all local repositories that match a pattern
  branch    Set default, set protected branch
  checkout  Checkout a branch all repositories that their name matches a pattern or a topic
  ci        Generate or export ci configuration
  clone     Clone all repositories that matches a pattern
  clean     Do git clean -f for all local repositories that match a pattern
  commit    Add all and then commit with the provided messages for all repositories that match a pattern or a topic
  create    Create a team, discussion, repository, or branch
  fetch     Fetch all local repositories that match a regex
  health    Comprehensive health check for repositories
  hook      Create or delete hooks for all repositories that match a pattern
  init      Init configuration data
  invite    Invite users to an organisation by email
  label     List, create, delete or rename labels
  make      Make repositories that match a regex become public/private
  merge     Merge a branch into the current branch for all repositories that match a pattern
  pull      Pull the current branch of all local repositories that match a regex
  push      Push the provided branch to remote server for all repositories that match a pattern or a topic
  remove    Remove users (from an organisation or team) or delete repositories
  rename    Rename repositories or teams
  rename
    repo          Rename a repository
    team          Rename a team in an organisation
  set       Set information, secret for repositories or permission for a team
  show      Show config, repositories, members, or user access
  status    Show git status of all repositories that match a pattern
  template  Apply changes or generate new template
  topic     Add, get, set or apply a script by topic
  transfer  Transfer repositories that match a regex to another organisation
  workflow  Run a workflow
  help      Print this message or the help of the given subcommand(s)

Options:
      --format <FORMAT>  [default: table] [possible values: table, json]
  -h, --help             Print help (see more with '--help')
  -V, --version          Print version
```

### Some subcommands Reference

Commands with subcommands:

```
add
    repos         Add all matched repositories to a team by using team_slug
    users         Invite users by users' usernames to an organisation

branch
    default       Set a branch as default for all repositories that match a pattern
    protect       Set a branch as protected for all local repositories that match a pattern
    unprotect     Remove branch protection for all local repositories that match a pattern

ci
    export        Export data file for ci generate command
    generate      Generate ci for every repositories that matches

create
    branch        Create a new branch for all repositories that match a regex or a topic
    discussion    Create a discussion for a team in an organisation
    repo          Create new repositories in an organisation and push for existing git repositories
    team          Create a new team for an organisation

hook
    create        Create web hook for repos matching regex
    delete        Delete ALL web hooks for all repositories that match given regex

invite
    users         Invite users to an organisation by emails

label
    list          List labels for all repositories that match a regex
    create        Create a label for all repositories that match a regex
    delete        Delete a label from all repositories that match a regex
    rename        Rename a label for all repositories that match a regex

remove
    repositories  Delete repositories matching a pattern
    users         Remove users by users' usernames from an organisation

set
    info          Set description and/or website for all repositories that match regex
    owner         Set default owner (organisation or user) name for every other command
    permission    Set access permissions for a team on repos matching regex
    secret        Set a secret for all repositories that match regex

show
    config        Show current configuration
    repositories  Show all repositories that match a pattern [aliases: repos]
    repository    Show details for a specific repository (teams, collaborators)
    access        Show repositories accessible by specified user(s) in an organisation
    members       Show all members in an organisation [aliases: users]
    teams         Show all teams in an organisation
    team          Show details of a specific team

template
    apply         Apply changes from template to all projects that match the regex
    generate      Generate a new project from a template
    refresh       Refresh placeholder substitutions in files based on .gut/delta.toml

topic
    add           Add topics for all repositories that match a regex
    apply         Apply a script to all repositories that have topics matching a pattern
    list          List topics for all repositories that match a regex
    set           Set topics for all repositories that match a regex

workflow
    run           Rerun the most recent workflow or send a repository_dispatch event
```
