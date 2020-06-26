# gut

![](https://github.com/divvun/gut/workflows/Gut%20Check/badge.svg)

This is a Git(Hub) multirepo maintenance tool, designed specifically for Divvun. But it should be quite useful to others needing to maintain tens (or hundreds) of similarly structured github repositories.

Using the `gut apply -s <script>` command, one can in practice run any git command on all repos (or a suitable subset, regex-selected on the repo names), not only the commands directly provided by `gut`.

We think it's pretty cool.

## Installation

1. get [Rust](https://www.rust-lang.org/learn/get-started)
1. clone this repo: `git clone https://github.com/divvun/gut.git`
2. `cd gut`
1. `cargo install --path .`

## Setup

1. make a [personal access token](https://github.com/settings/tokens) in GitHub - allow everything. Make sure to store it in a safe place - the token replaces your username and password when accessing GitHub via `gut`.
1. run `gut init -r <root-dir> -t <token>`

`<token>` is the token created in step 1. above.
The `<root-dir>` is the directory where you want to store all repos processed by `gut`.
Below the `<root-dir>` dir, there will be one directory for every organisation you interact with, and within the organisation directory all repos are stored.

## Usage

There are some [usage instructions](USAGE.md) under development.

Then there are some use cases with example commands
[here](https://github.com/divvun/giellalt-svn2git/blob/master/doc/GutUsageExamples.md).

There is also some rudimentary help text. Run `gut --help` to get an overview.
