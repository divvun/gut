# USAGE

## Add Users

```
    -o, --organisation <organisation>     [default: divvun]
    -r, --role <role>                     [default: member]
    -t, --team-slug <team-slug>
    -u, --users <users>...
```

Users must be a space separated list of GitHub user ID's, possibly also multiple `-u` options.

### Effect

Sends an invitation request to the specified user(s), to become a member of the specified organisation.

## Invite Users

```
dadmin-invite-users 0.1.0
Invite users to an organisation by emails

USAGE:
    dadmin invite users [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --emails <emails>...             list of user's emails
    -o, --organisation <organisation>    Target organisation name [default: divvun]
    -r, --role <role>                    Role of users It should be one of ["member", "admin", "billing_manager"]
                                         [default: member]
```

### Effect

Invite users by emails.

## Merge

`dadmin merge -o <org> -r <regex> --branch <branch> --abort-if-conflict`

### Effect

This command will try to merge a branch into your head branch for all repositories that match a regex pattern.

This works similar to `git merge` command. Dadmin will use `fast-forward` strategy whenever possible.

If there is a conflict, that it cannot resolve automatically, you'll need to fix all conflicts and then commit it yourself. Or you can use `--abort-if-conflict` option to abort it.

Dadmin also shows all merge conflict files as normal `git merge` command.

If you want to merge a branch `A` into branch `B`, you can check out branch `B` first and then use this merge command.

## Clean

`dadmin clean -o <org> -r <regex>`

### Effect

This command will try to simulate `git clean -f -d` command. It will clean all local repositories that match a regex pattern.

## Status

`dadmin status -o <org> -r <regex> --verbose`

### Effect

This command will try to show statuses of all local repositories that match a regex pattern.

## Commit

`dadmin commit -o <org> -r <regex> --message <message>`

### Effect

This command will add all changes and create a commit with the provided message for all local repositories that match a regex pattern.

If there is any conflict, this will be aborted.
If there is no changes, this also will be aborted.
