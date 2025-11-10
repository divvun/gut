---
icon: lucide/git-branch
---

# gut

**Pronounced** `gʉːt`

gut is a **Git(Hub) multirepo maintenance tool** designed for managing tens or hundreds of similarly structured GitHub repositories. While built specifically for [Divvun](https://divvun.no/), it's useful for any organization maintaining multiple repositories.

## What is gut?

gut makes it easy to perform operations across many repositories at once. Using the `gut apply -s <script>` command, you can run any git command on all repos (or a suitable subset, regex-selected by repo names) - not just the commands directly provided by `gut`.

## Key Features

- **Bulk operations** - Clone, fetch, pull, push, commit across multiple repos
- **Flexible filtering** - Select repos by regex pattern or GitHub topics
- **Script execution** - Run custom scripts across all matching repositories
- **GitHub integration** - Manage teams, permissions, secrets, webhooks
- **Repository management** - Set descriptions, visibility, default branches
- **CI/CD integration** - Generate and manage CI configurations
- **Template system** - Apply consistent changes across repositories

## Use Cases

- Updating dependencies across all projects
- Applying security patches to multiple repos
- Standardizing CI/CD configurations
- Managing team permissions and access
- Bulk repository migrations or transfers
- Enforcing branch protection rules
- Setting up webhooks and secrets

## Quick Example

```bash
# Clone all repositories matching a pattern
gut clone -o myorg -r "lang-.*"

# Check status across all repos
gut status -o myorg -r ".*"

# Apply a script to all matching repos
gut apply -o myorg -r "project-.*" -s ./update-deps.sh

# Set a secret for all repos with a specific topic
gut topic apply -o myorg -t "backend" -s ./deploy-secret.sh
```

## Getting Started

Ready to get started? Head over to the [Get Started](get-started.md) guide to install gut and set up your environment.

## Community

- **Repository**: [github.com/divvun/gut](https://github.com/divvun/gut)
- **Issues**: [Bug reports & feature requests](https://github.com/divvun/gut/issues)
- **Downloads**: [Nightly builds](https://pahkat.uit.no/devtools/download/gut?channel=nightly)
