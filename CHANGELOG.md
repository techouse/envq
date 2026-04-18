# Changelog

All notable changes to envq are documented here.

## 0.1.1 - 2026-04-18

- Added crate homepage metadata for package registries.
- Declared the `envq` binary target explicitly in `Cargo.toml`.
- Added generated third-party dependency license notices for binary distributions.
- Bundled `THIRD-PARTY-LICENSES.md` in Linux, macOS, and Windows release archives.
- Installed third-party license notices under `/usr/share/doc/envq/` in Linux `.deb` and `.rpm` packages.
- Trimmed crates.io package contents by excluding release-maintainer files, third-party notice generation files, and repository-only attributes.

## 0.1.0 - 2026-04-17

- Initial Rust release of `envq`.
- Added byte-preserving parsing, rendering, editing, and CLI support for the documented `.env` subset.
- Preserves comments, spacing, duplicate keys, invalid UTF-8 bytes, and newline styles across unrelated edits.
- Supports `get`, `has`, `list`, `set`, `clear`, `unset`, `remove`, `help`, and `completion` commands.
- Provides generated completions for bash, zsh, fish, PowerShell, and pwsh.
- Includes golden fixture tests, local fuzz targets, and cross-platform CI.
