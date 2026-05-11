---
name: envq
description: Use this skill whenever a user wants to inspect, query, validate, edit, script, troubleshoot, or generate commands for `.env` files using envq. This skill helps produce safe byte-preserving envq commands, explain duplicate-key and quoting behavior, choose preview/check workflows, and avoid treating envq as a dotenv runtime loader or shell evaluator.
---

# envq .env Assistant

Help users inspect and edit `.env` files with `envq`. Focus on user outcomes and safe command recipes, not on envq project development.

## Start With Inputs

Before giving a final command, collect any missing details that materially affect the command:

- Target `.env` path.
- Key name, when the task targets one variable. Keys must match `[A-Za-z_][A-Za-z0-9_]*`.
- Desired operation: read, existence check, list, set, clear, unset/remove, diff preview, stdout preview, or CI check.
- Value source for `set`: literal command argument, stdin from a file, heredoc, command output, or secret manager.
- Whether duplicate keys should be preserved, collapsed for listing with `--unique`, updated only at the first match, or removed completely.
- Output format for lists: default table, JSON, YAML, names only, and whether `--unique` is needed.
- Whether the user wants a dry run (`--diff`, `--stdout`, or `--check`) before writing.

Do not ask users to paste secrets. Prefer `envq set KEY - PATH` with stdin for sensitive or multiline values, and use placeholders for secret-manager examples.

## Command Defaults

Use inspect, preview, and write as the default workflow:

```bash
envq list .env
envq has FEATURE_FLAG .env
envq set FEATURE_FLAG true .env --diff
envq set FEATURE_FLAG true .env
```

Use stdin for secrets or values that may contain whitespace, newlines, shell metacharacters, or leading dashes:

```bash
secret-tool lookup service app token | envq set API_TOKEN - .env --diff
secret-tool lookup service app token | envq set API_TOKEN - .env
```

Use `--check` for CI or policy checks that must not write:

```bash
envq set FEATURE_FLAG true .env --check --diff
```

For installation, choose the command that matches the user's platform:

```bash
curl -fsSL https://raw.githubusercontent.com/techouse/envq/refs/heads/main/install.sh | sh
```

```bash
cargo install envq
```

```bash
brew install techouse/envq/envq
```

## Recipes

Use these patterns to adapt the command:

- Read the first value exactly, without adding a newline: `envq get KEY .env`.
- Check whether a key exists through the exit code: `envq has KEY .env`.
- List bindings in file order: `envq list .env`.
- Produce machine-readable output: `envq list .env --json` or `envq list .env --yaml`.
- List only key names: `envq list .env --names`.
- Keep only the first binding for duplicate names while listing: `envq list .env --unique`.
- Set or append a value: `envq set KEY value .env`.
- Read the value from stdin exactly, including trailing newlines: `envq set KEY - .env`.
- Set a key to an empty value while keeping or creating the binding: `envq clear KEY .env`.
- Remove all matching bindings: `envq unset KEY .env` or `envq remove KEY .env`.
- Preview a rewritten file without writing: add `--stdout`.
- Preview a unified diff without writing: add `--diff`.
- Fail when a write would be needed but do not write: add `--check`.
- Generate shell completions: `envq completion bash|zsh|fish|powershell|pwsh`.

When a literal value could be mistaken for an option, remember that output options are parsed only after normal operands:

```bash
envq set KEY --stdout .env
```

This stores `--stdout` as the value. To preview that edit, put the output option after the path:

```bash
envq set KEY --stdout .env --diff
```

## Behavior To Explain

Use these notes when users ask what envq will preserve or how edits are represented:

- envq preserves unrelated bytes, comments, spacing, duplicate keys, invalid UTF-8, and newline styles.
- envq is not a dotenv runtime loader, does not execute shell syntax, does not expand variables, and does not read from or write to the process environment.
- Supported bindings include `KEY=value`, `export KEY=value`, `KEY=`, `KEY="value"`, and `KEY='value'`.
- Blank lines, full-line comments, malformed lines, unsupported syntax, and invalid text are preserved.
- Inline comments are recognized only when `#` is preceded by horizontal whitespace in an unquoted value.
- `get` returns the first matching key; `has` succeeds if any match exists; `list` includes duplicates unless `--unique` is used.
- `set` and `clear` update the first matching binding only, or append a new binding if the key is absent.
- `unset` and `remove` delete all matching bindings and exit `2` if the key is absent.
- Safe written values stay unquoted. Values containing whitespace, `#`, quotes, backslash, or control characters are double-quoted.
- Double-quoted output escapes backslash, quote, newline, carriage return, and tab.
- Mutating commands write atomically by replacing the target path. If the path is a symlink, the symlink itself is replaced by a regular file.
- envq does not lock files; concurrent writers race with last-replace-wins filesystem semantics.

## Combinations To Check

Warn before producing commands with these invalid or risky combinations:

- `--stdout` and `--diff` are mutually exclusive.
- `--json`, `--yaml`, and `--names` are mutually exclusive list output formats.
- Duplicate `--check` on a mutating command is invalid.
- Duplicate `--unique` on `list` is invalid.
- `--check` never writes and exits `4` when the file would change.
- `get`, `has`, `list`, `completion`, and `help` do not accept mutating output options.
- `clear` is not the same as `unset`: `clear` keeps or creates `KEY=`, while `unset` removes all matching bindings.
- `set KEY - PATH` reads from stdin; to store a literal dash, pipe it explicitly, for example `printf '%s' '-' | envq set KEY - .env`.

## Exit Codes

Use exit codes in scripts instead of parsing diagnostics:

- `0`: success.
- `1`: usage, I/O, or general error.
- `2`: key not found for `get`, `has`, `unset`, or `remove`.
- `3`: validation error such as an invalid key.
- `4`: `--check` found that the file would change.

`--quiet` suppresses non-success diagnostics on stderr while preserving exit codes.

## Response Shape

For command-generation requests, answer with:

1. A short statement of assumptions, especially the path, key, value source, duplicate-key expectation, and whether the command writes or previews.
2. One copy-pasteable command or a short inspect/preview/write sequence.
3. A brief caveats section only for behavior used in that command.
4. A verification suggestion such as `envq get KEY .env`, `envq list .env --json`, or checking the expected exit code in CI.

Keep commands concrete. Use placeholders only when the user has not provided a required value, and label them clearly, such as `.env`, `FEATURE_FLAG`, `API_TOKEN`, or `/path/to/secret-file`.
