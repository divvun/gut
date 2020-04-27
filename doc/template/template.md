# Template

## Requirement

- Generate a new repository Y from a template X

- Apply changes from a template X to a repo Y that generated from X
  - This action needs to support new added files and edited files.
  - Moved, and deleted files also need to support but this will be lower priority.
  - Differentiating between safe and unsafe files.

- Support old templates and old generated repos (from svn)

## Solution a.k.a. Git patch sets via templating

### General idea

We will define a `delta` file which stores all the information we need to process our actions (generating new repos or applying changes).

The structure of `delta` file for a template and a repository will be similar but contain some difference.

In order to generate a new repo, we need to create a new `delta` and then use it to generate the new repo.

In order to apply changes from template. We need to read `delta` file from both the target repo and the template to define some data like: The last time the target repo got updated? Which files need to change? How to determine which files in the target repo correspond to which files in the template.

### List of commands

Let says `dadmin template` is our sub command for this task. Below are our list of commands:
The bold one is nesscesary for our current task (apply changes).

#### Template

- `dadmin template init`: Init delta file for a template repo
- `dadmin template bump-version`: Mark the current version of the template is the newest version of template
- `dadmin template {add,remove} [--optional|--ignore]`: Adds or remove a file from the template list, with flags for adding to the optional or ignored list
- `dadmin template pattern {list,add,remove}`: Manage file patterns (like UND -> some lang)

#### Repo

- `dadmin generate-repo <template-url>`: Create a new repository from a template url
- `dadmin template install <template-url>`: Init delta file for a repo
- `dadmin template apply [--continue|--abort]`: Start apply changes from template - if conflict, running with --continue after starting will allow finishing it, and --abort will revert any patching changes.
- `dadmin template replacement {set,remove}`: Add or remove replacements from the template file

### Delta file structure

Please read samples of delta files (which have some comments).

### How do we do apply changes?

1. Run `dadmin template apply`
2. Mark this process as in applying process
3. Read delta files of both template and target repo.
4. Determine the last changes (use rev_id)
5. Get the diff of template from last changes (by comparing the diff between the newest commit sha to the current commit's sha of target repo)
6. Use search/replace on list of patterns in the diff content
7. Apply the diff we got from last step to the target repo (use list of patterns to determine the corressponding files).
8. Show status of this action (changes, conflicts, etc)
9. Manually resolve conflict if needed
10. Manually commit
11. Run `dadmin template apply --continue` => Mark this process is done.
