# USAGE

## Add Users

```
gut-add-users 0.1.0
Invite users to an organisation by usernames.

If you specify team_slug it'll try to invite users to the provided team

USAGE:
    gut add users [OPTIONS]

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
gut-invite-users 0.1.0
Invite users to an organisation by emails

USAGE:
    gut invite users [OPTIONS]

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

`gut merge -o <org> -r <regex> --branch <branch> --abort-if-conflict`

### Effect

This command will try to merge a branch into your head branch for all repositories that match a regex pattern.

This works similar to `git merge` command. gut will use `fast-forward` strategy whenever possible.

If there is a conflict, that it cannot resolve automatically, you'll need to fix all conflicts and then commit it yourself. Or you can use `--abort-if-conflict` option to abort it.

gut also shows all merge conflict files as normal `git merge` command.

If you want to merge a branch `A` into branch `B`, you can check out branch `B` first and then use this merge command.

## Clean

`gut clean -o <org> -r <regex>`

### Effect

This command will try to simulate `git clean -f -d` command. It will clean all local repositories that match a regex pattern.

## Status

`gut status -o <org> -r <regex> --verbose`

### Effect

This command will try to show statuses of all local repositories that match a regex pattern.

## Commit

`gut commit -o <org> -r <regex> --message <message>`

### Effect

This command will add all changes and create a commit with the provided message for all local repositories that match a regex pattern.

If there is any conflict, this will be aborted.
If there is no changes, this also will be aborted.

## Fetch

```
gut-fetch 0.1.0
Fetch all local repositories that match a regex

USAGE:
    gut fetch [OPTIONS]

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
gut-make 0.1.0
Make repositories that match a regex become public/private

This will show all repositories that will affected by this command If you want to public repositories, it'll show a
confirmation prompt and You have to enter 'YES' to confirm your action

USAGE:
    gut make [OPTIONS] --regex <regex> <SUBCOMMAND>

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
gut-set-info 0.1.0
Set description and/or website for all repositories that match regex

Description can be provided by --description option or --des-script option

When it is provided --des-script will override --description

Similar to --web-script and --website

USAGE:
    gut set info [OPTIONS] --regex <regex>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --des-script <des-script>        The script that will produce a description
    -d, --description <description>      Description, this is required unless website is provided
    -o, --organisation <organisation>    Target organisation name [default: divvun]
    -r, --regex <regex>                  Optional regex to filter repositories
        --web-script <web-script>        The script that will produce a website
    -w, --website <website>              Homepage, this is required unless description is provided
```

The script can use two arguments: repository name as argument number one organisation name as argument number two.

Here is a sample of a description scrip
```
name=$1
org=$2
printf "This is the best description ever for ${name} in ${org}"
```

## Set secret

```
gut-set-secret 0.1.0
Set a secret all repositories that match regex

USAGE:
    gut set secret [OPTIONS] --name <name> --regex <regex> --value <value>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -n, --name <name>                    The name of your secret
    -o, --organisation <organisation>    Target organisation name [default: divvun]
    -r, --regex <regex>                  Optional regex to filter repositories
    -v, --value <value>                  The value for your secret
```

## Topic

```
gut-topic 0.1.0
Sub command for set/get/add topics

USAGE:
    gut topic <SUBCOMMAND>

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
gut-topic-get 0.1.0
Get topics for all repositories that match a regex

USAGE:
    gut topic get [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --organisation <organisation>    Target organisation name [default: divvun]
    -r, --regex <regex>                  Optional regex to filter repositories

```

### Topic Set

```
gut-topic-set 0.1.0
Set topics for all repositories that match a regex

USAGE:
    gut topic set [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --organisation <organisation>    Target organisation name [default: divvun]
    -r, --regex <regex>                  Optional regex to filter repositories
    -t, --topics <topics>...             All topics will be set
```

## Topic Add

```
gut-topic-add 0.1.0
Add topics for all repositories that match a regex

USAGE:
    gut topic add [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --organisation <organisation>    Target organisation name [default: divvun]
    -r, --regex <regex>                  Optional regex to filter repositories
    -t, --topics <topics>...             All topics will be added
```

## Topic Apply

```
gut-topic-apply 0.1.0
Apply a script to all repositories that has a topics that match a pattern Or to all repositories that has a specific
topic

USAGE:
    gut topic apply [FLAGS] [OPTIONS] --regex <regex> --script <script> --topic <topic>

FLAGS:
    -h, --help         Prints help information
    -u, --use-https    use https to clone repositories if needed
    -V, --version      Prints version information

OPTIONS:
    -o, --organisation <organisation>    Target organisation name [default: divvun]
    -r, --regex <regex>                  regex pattern to filter topics. This is required unless topic is provided
    -s, --script <script>                The script will be applied for all repositories that match
    -t, --topic <topic>                  A topic to filter repositories. This is required unless regex is provided
```

## Hook Create

```
gut-hook-create 0.1.0

USAGE:
    gut hook create [OPTIONS] --method <method> --regex <regex> --script <script> --url <url>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --events <events>...             Determines what events the hook is triggered for
    -m, --method <method>                Content type, either json or form
    -o, --organisation <organisation>    Target organisation name [default: divvun]
    -r, --regex <regex>                  Optional regex to filter repositories
    -s, --script <script>                The script that will produce an url
    -u, --url <url>                      The url to which payloads will be delivered
```
