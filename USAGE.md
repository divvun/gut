# USAGE

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

`dadmin status -o <org> -r <regex>`

### Effect

This command will try to show statuses of all local repositories that match a regex pattern.
