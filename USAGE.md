# USAGE

## Add Users

```
dadmin-add-users 0.1.0
Invite users to an organisation by usernames.

If you specify team_slug it'll try to invite users to the provided team

USAGE:
    dadmin add users [OPTIONS]

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information


OPTIONS:
    -o, --organisation <organisation>
            Target organisation name [default: divvun]

    -r, --role <role>
            Role of users

            It should be one of ["member", "admin"].

            If you specify a team role should be one of ["member", "maintainer"] [default: member]
    -t, --team-slug <team-slug>
            optional team slug

    -u, --users <users>...
            list of user's usernames

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

## Fetch

```
dadmin-fetch 0.1.0
Fetch all local repositories that match a regex

USAGE:
    dadmin fetch [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --organisation <organisation>    Target organisation name [default: divvun]
    -r, --regex <regex>                  Optional regex to filter repositories

```

## Make

Change visibilities of repositories

```
dadmin-make 0.1.0
Make repositories that match a regex become public/private

This will show all repositories that will affected by this command If you want to public repositories, it'll show a
confirmation prompt and You have to enter 'YES' to confirm your action

USAGE:
    dadmin make [OPTIONS] --regex <regex> <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --organisation <organisation>    Target organisation name [default: divvun]
    -r, --regex <regex>                  Regex to filter repositories

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    private
    public
```

## Set info

```
dadmin-set-info 0.1.0
Set description and/or website for all repositories that match regex

USAGE:
    dadmin set info [OPTIONS] --regex <regex>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --description <description>      Description, this is required unless website is provided
    -o, --organisation <organisation>    Target organisation name [default: divvun]
    -r, --regex <regex>                  Optional regex to filter repositories
    -w, --website <website>              Hompage, this is required unless description is provided
```

## Topic

```
dadmin-topic 0.1.0
Sub command for set/get/add topics

USAGE:
    dadmin topic <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    add     
    get     
    help    Prints this message or the help of the given subcommand(s)
    set     Set topics for all repositories that match a regex

```

### Topic Get

```
dadmin-topic-get 0.1.0
Get topics for all repositories that match a regex

USAGE:
    dadmin topic get [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --organisation <organisation>    Target organisation name [default: divvun]
    -r, --regex <regex>                  Optional regex to filter repositories

```

### Topic Set

```
dadmin-topic-set 0.1.0
Set topics for all repositories that match a regex

USAGE:
    dadmin topic set [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --organisation <organisation>    Target organisation name [default: divvun]
    -r, --regex <regex>                  Optional regex to filter repositories
    -t, --topics <topics>...             All topics will be set
```

## Topic add

```
dadmin-topic-add 0.1.0
Add topics for all repositories that match a regex

USAGE:
    dadmin topic add [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --organisation <organisation>    Target organisation name [default: divvun]
    -r, --regex <regex>                  Optional regex to filter repositories
    -t, --topics <topics>...             All topics will be added
```
