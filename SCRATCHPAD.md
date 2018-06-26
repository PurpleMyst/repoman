# gitman

`gitman` will be a CLI with the following sub-commands:

- `gitman new PROJECT-NAME [--from TEMPLATE-NAME]` creates a new project from a template.

- `gitman label` sub-command:
    - `gitman label add LABEL` adds a label to the current project.
    - `gitman label remove LABEL` removes a label from the current project.

- `gitman batch` sub-command:
    - `gitman batch LABEL COMMAND [ARG ...]` runs a batch operation on all git
      projects with the `LABEL` label. The command's working directory is the
      top-level directory of each repo.
