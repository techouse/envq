# Changelog

All notable changes to envq are documented here.

## 0.1.0 - 2026-04-17

- Initial Rust release of `envq`.
- Added byte-preserving parsing, rendering, editing, and CLI support for the documented `.env` subset.
- Preserves comments, spacing, duplicate keys, invalid UTF-8 bytes, and newline styles across unrelated edits.
- Supports `get`, `has`, `list`, `set`, `clear`, `unset`, `remove`, `help`, and `completion` commands.
- Provides generated completions for bash, zsh, fish, PowerShell, and pwsh.
- Includes golden fixture tests, local fuzz targets, and cross-platform CI.
