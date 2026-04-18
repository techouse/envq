# Copilot Project Instructions: envq.rs

Concise, project-specific guidance for AI coding agents working on this repo. Focus on preserving envq compatibility with the documented behavior and golden fixtures while keeping the Rust implementation byte-safe and small.

## 1. Project Purpose And Architecture
- Library and binary: Rust implementation of the `envq` CLI and library.
- Current crate-wide MSRV: Rust `1.88`.
- `src/main.rs` is a thin process wrapper. Core behavior lives in `src/lib.rs`.
- Internal modules:
  - `src/model.rs`: raw byte document model and enums.
  - `src/parser.rs`: `.env` parser and value decoding.
  - `src/render.rs`: byte-for-byte rendering and value quoting.
  - `src/editor.rs`: pure document edits.
  - `src/cli.rs`: command parsing, diagnostics, and command execution.
  - `src/diff.rs`: unified diff output matching the documented fixture shape.
  - `src/io_atomic.rs`: same-directory temp file writes and atomic replace.
  - `src/diagnostics.rs`: shared exit codes and diagnostics.
- Compatibility fixtures live under `tests/fixtures/golden/`.

## 2. Behavioral Oracles
- Source of truth order:
  1. `README.md` behavior contract
  2. golden fixtures in `tests/fixtures/golden/`
- Keep any legacy reference comparisons ignored by default. They are maintainer-only sweeps, not required local tests.

## 3. Key Invariants
- The parser, editor, renderer, CLI output, and file I/O are byte-backed. Do not convert document contents to `String` except for validated keys or explicitly UTF-8-only metadata.
- Invalid UTF-8 must survive unrelated edits and list output must remain byte-compatible with the golden fixtures.
- Keys are ASCII and match `[A-Za-z_][A-Za-z0-9_]*`.
- Preserve duplicate semantics: first match for `get`, `set`, and `clear`; all matches for `unset` and `remove`.
- Preserve existing prefixes, spacing, suffixes, inline comments, and line terminators when updating bindings.
- Mutation output options are intentionally position-sensitive. For example, `envq set KEY --stdout PATH` stores `--stdout` as the value.
- Atomic writes should preserve existing POSIX mode where supported and replace symlink paths with regular files.

## 4. Developer Workflow
- Required local checks for substantial changes:
  - `cargo +1.88.0 fmt --check`
  - `cargo +1.88.0 clippy --all-targets --all-features -- -D warnings`
  - `cargo +1.88.0 test --locked`
  - `cargo test --locked`
- Optional legacy compatibility sweep when changing CLI, rendering, parser, or file behavior:
  - `ENVQ_LEGACY_REFERENCE=<checkout> ENVQ_LEGACY_REFERENCE_RUNNER=<runner> cargo test --test legacy_reference -- --ignored`
- Packaging check before release or CI changes:
  - `cargo package --locked`

## 5. Testing Strategy
- Add focused Rust unit tests for narrow parser/render/editor rules.
- Add or update golden fixtures for user-visible behavior.
- Use `.escaped` sidecar files for newline-sensitive or invalid-byte data.
- Normalize only temporary paths in differential tests. Do not normalize exit codes, rewritten file bytes, stdout bytes, or stderr bytes.

## 6. Common Pitfalls
- Accidentally making document storage `String`-backed.
- Treating undocumented quirks as contract without adding fixtures.
- Losing invalid UTF-8 bytes while formatting JSON/YAML-like list output.
- Letting `clap`-style trailing option parsing override envq's operand-sensitive mutation rules.
- Adding broad dotenv syntax not documented by the README behavior contract.

## 7. When Unsure
- Check the README behavior contract first.
- Check existing golden fixtures before changing behavior.
- Run the ignored legacy compatibility sweep if the behavior is CLI-visible and the legacy reference checkout is available.
- Prefer the smallest compatibility-preserving change over new abstractions.

---
If these instructions conflict with measured behavior, tests, or the README behavior contract, follow the measured/tested behavior and update the docs or fixtures explicitly.
