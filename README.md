# gut

[![](https://divvun-tc.thetc.se/api/github/v1/repository/divvun/gut/main/badge.svg)](https://divvun-tc.thetc.se/api/github/v1/repository/divvun/gut/main/latest)

This is a Git(Hub) multirepo maintenance tool, designed specifically for Divvun. But it should be quite useful to others needing to maintain tens (or hundreds) of similarly structured github repositories.

Using the `gut apply -s <script>` command, one can in practice run any git command on all repos (or a suitable subset, regex-selected on the repo names), not only the commands directly provided by `gut`.

We think it's pretty cool.


## Installation

1. get [Rust](https://www.rust-lang.org/learn/get-started)
1. clone this repo: `git clone https://github.com/divvun/gut.git`
2. `cd gut`
1. `cargo install --path .`

**Alternatively** - download a precompiled binary from nightly builds:

* [Linux  ](https://pahkat.uit.no/devtools/download/gut?channel=nightly&platform=linux)   (x86_64)
* [macOS  ](https://pahkat.uit.no/devtools/download/gut?channel=nightly&platform=macos)   (x86_64)
* [Windows](https://pahkat.uit.no/devtools/download/gut?channel=nightly&platform=windows) (i686)

Extract the archive, and move the binary to somewhere on your `$PATH`.

## Setup

1. make a [personal access token](https://github.com/settings/tokens) in GitHub - allow everything. Make sure to store it in a safe place - the token replaces your username and password when accessing GitHub via `gut`.
1. run `gut init -r <root-dir> -t <token>`

`<token>` is the token created in step 1. above.
The `<root-dir>` is the directory where you want to store all repos processed by `gut`.
Below the `<root-dir>` dir, there will be one directory for every organisation you interact with, and within the organisation directory all repos are stored.

### SSH access over the `git` protocoll

To use the `git`/`ssh` protocol, you need to set up an `ssh` key for GitHub. Follow [these instructions](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/generating-a-new-ssh-key-and-adding-it-to-the-ssh-agent).

## Usage

> **NB!** Please note that this is a potentially very powerful tool. Some commands require *organisation owner permissions*, and the most dangerous ones will require an *explicit confirmation*. If you get an error that the operation is not permitted, you probably do not have sufficient access to the repos involved.

There are some [usage instructions](USAGE.md) under development.

Then there are some use cases with example commands
[here](https://giellalt.github.io/infra/infraremake/HowToMergeUpdatesFromCore.html).

There is also some rudimentary help text. Run `gut --help` to get an overview.

In version 0.1.0 it reads:

```
$ gut --help         
gut 0.1.0
git multirepo maintenance tool

USAGE:
    gut <SUBCOMMAND>

FLAGS:
    -h, --help       
            Prints help information

    -V, --version    
            Prints version information


SUBCOMMANDS:
    add         Add users, repos to an organisation/a team
    apply       Apply a script to all local repositories that match a pattern
    branch      Set default, set protected branch
    checkout    Checkout a branch all repositories that their name matches a pattern or a topic
    ci          
    clean       Do git clean -f for all local repositories that match a pattern
    clone       Clone all repositories that matches a pattern
    commit      Add all and then commit with the provided messages for all repositories that match a pattern or a topic
    create      Create team, discussion, repo to an organisation or create a branch for repositories
    fetch       Fetch all local repositories that match a regex
    help        Prints this message or the help of the given subcommand(s)
    hook        Create, delete hooks for all repositories that match a pattern
    init        Init configuration data
    invite      Invite users to an organisation by emails
    make        Make repositories that match a regex become public/private
    merge       Merge a branch to the current branch for all repositories that match a pattern
    pull        Pull the current branch of all local repositories that match a regex
    push        Push the provided branch to remote server for all repositories that match a pattern or a topic
    remove      Remove users, repos from an organisation/a team
    set         Set information, secret for repositories or permission for a team
    show        Show config, list of repositories or users
    status      Show git status of all repositories that match a pattern
    template    Apply changes or generate new template
    topic       Add, get, set or apply a script by topic
    transfer    Transfer repositories that match a regex to another organisation
    workflow    Run a workflow
```

Help text for subcommands with further details reads:

```
SUBCOMMANDS with additional arguments:
    add
        repos       - Add all matched repositories to a team by using team_slug
        users       - Invite users by users' usernames to an organisation
    branch
        default     - Set a branch as default for all repositories that match a pattern
        protect     - Set a branch as protected for all local repositories that match a pattern
    ci          Continuous Integration
        export      - export data file for ci generate command
        generate    - generate ci for every repositories that matches
    create      Create team, discussion, repo to an organisation or create a branch for repositories
        branch      - Create a new branch for all repositories that match a regex or a topic
        discussion  - Create a discussion for a team in an organisation
        repo        - Create new repositories in an organisation and push for existing git repositories
        team        - Create a new team for an organisation
    hook        Create, delete hooks for all repositories that match a pattern
        create      - Create web hook for repos matching regex
        delete      - Delete all web hooks for all repository that match regex
    invite      Invite users to an organisation by emails
        users       - Invite users to an organisation by emails
    make        Make repositories that match a regex become public/private
        private    
        public     
    remove      Remove users, repos from an organisation/a team
        repositories    
        users       - Remove users by users' usernames from an organisation
    set         Set information, secret for repositories or permission for a team
        info        - Set description and/or website for all repositories that match regex, plain text or using a script
                      NB! Make sure there is no trailing newline at the end! Or it will fail.
        organisation- Set default organisation name for every other command
        permission  - Set access permissions for a team, for repos matching regex; matching repos will be added if not already in the team
        secret      - Set a secret all repositories that match regex
    show        Show config, list of repositories or users
        config      - Print configuration
        repositories- Show all repos matching regex   
        users       - Show all users in an organisation
    template    Apply changes or generate new template
        apply       - Apply changes from template to all repos that match the regex
        generate    - Generate a new project from a template
    topic       Add, get, set or apply a script by topic
        add      Add topics for all repositories that match a regex
        apply    Apply a script to all repositories that has a topics that match a pattern Or to all repositories that has a specific topic
        get      Get topics for all repositories that match a regex
        set      Set topics for all repositories that match a regex
    workflow    Run a workflow
        run         - Rerun the most recent workflow or send a repository_dispatch event to trigger workflows
```
