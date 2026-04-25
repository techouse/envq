# Changelog

All notable changes to envq are documented here.

## 0.1.3 - 2026-04-25

- Bundled bash, zsh, and fish completions in Linux GNU/glibc and musl `.tar.gz` release archives.
- Installed bash, zsh, and fish completions by default from `install.sh`, with `ENVQ_INSTALL_COMPLETIONS=0` available to skip completion installation.
- Added installer smoke coverage for packaged completions, generated completion fallback, opt-out behavior, and non-fatal completion install warnings.

## 0.1.2 - 2026-04-18

- Reached full source coverage for the shipped crate in the maintained coverage target.
- Hardened defensive diff fallback paths used when the diff engine input limits are exceeded.
- Hardened PowerShell completion patching so unexpected non-UTF-8 generator output is preserved instead of cleared.
- Added regression coverage for internal diff and completion edge cases without changing CLI behavior.

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
