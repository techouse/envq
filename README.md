# envq

Byte-preserving `.env` query and editing tool.

[![GitHub Release](https://img.shields.io/github/v/release/techouse/envq?logo=github)](https://github.com/techouse/envq/releases/latest)
[![Crates.io Version](https://img.shields.io/crates/v/envq?logo=rust)](https://crates.io/crates/envq)
[![Crates.io MSRV](https://img.shields.io/crates/msrv/envq?logo=rust)](https://crates.io/crates/envq)
[![Test](https://github.com/techouse/envq/actions/workflows/test.yml/badge.svg)](https://github.com/techouse/envq/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/techouse/envq/graph/badge.svg?token=E2nyGsMtBw)](https://codecov.io/gh/techouse/envq)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/ce234b7f752349f9a35a6904545b2aea)](https://app.codacy.com/gh/techouse/envq/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)
[![GitHub](https://img.shields.io/github/license/techouse/envq)](https://github.com/techouse/envq/blob/main/LICENSE)
[![GitHub Sponsors](https://img.shields.io/github/sponsors/techouse?logo=github)](https://github.com/sponsors/techouse)
[![GitHub Repo stars](https://img.shields.io/github/stars/techouse/envq)](https://github.com/techouse/envq/stargazers)

`envq` edits `.env` files deterministically while preserving unrelated bytes, comments,
spacing, duplicate keys, invalid UTF-8, and newline styles. It is not a dotenv
runtime loader, does not execute shell syntax, and does not read from or write to
the process environment.

The Rust implementation targets Rust 1.88. The CLI behavior documented below is
the compatibility contract.

## Library API

The CLI behavior documented in this README is the stable compatibility contract.
The Rust library modules are public for integration and testing during the `0.x`
series, but their exact API shape is still experimental and may change between
minor releases.

## Compatibility Contract

Implementations must match the golden fixtures byte-for-byte for rewritten files
and text-for-text for stdout, stderr, and exit codes. Platform-dependent
fixtures declare `platform` as `posix` or `windows`; all other fixtures declare
`platform` as `all`.

The golden fixture manifests in `tests/fixtures/golden/` are part of this
contract. Behavior that is not documented here or captured by those fixtures
should be treated as accidental until it is promoted into both places.

## Install

Install from crates.io:

```bash
cargo install envq
```

Install with Homebrew:

```bash
brew install techouse/envq/envq
```

Or download a release artifact from
[GitHub Releases](https://github.com/techouse/envq/releases). Linux releases
include GNU/glibc and musl `.tar.gz` archives for x86_64 and ARM64, plus
GNU/glibc `.deb` and `.rpm` packages for x86_64 and ARM64. Archives include
the `envq` binary, `README.md`, `LICENSE`, and `THIRD-PARTY-LICENSES.md`.
Verify downloads with the release `SHA256SUMS.txt` file or the per-artifact
`.sha256` sidecar.

Install shell completions by writing the generated script to your shell's
completion directory.

Bash:

```bash
mkdir -p ~/.local/share/bash-completion/completions
envq completion bash > ~/.local/share/bash-completion/completions/envq
```

Zsh:

```bash
mkdir -p ~/.zfunc
envq completion zsh > ~/.zfunc/_envq
```

Add this to `.zshrc` if `~/.zfunc` is not already in `fpath`:

```zsh
fpath=(~/.zfunc $fpath)
autoload -Uz compinit
compinit
```

Fish:

```fish
mkdir -p ~/.config/fish/completions
envq completion fish > ~/.config/fish/completions/envq.fish
```

PowerShell:

```powershell
New-Item -ItemType Directory -Force (Split-Path $PROFILE) | Out-Null
envq completion powershell | Add-Content $PROFILE
```

## Build

```bash
cargo build --locked
cargo build --release --locked
```

Run the development binary directly:

```bash
cargo run -- --help
cargo run -- --version
```

Or build once and run the binary:

```bash
cargo build --locked
./target/debug/envq --help
```

## Usage

```bash
envq [--version] [--quiet]
envq get KEY PATH
envq set KEY VALUE PATH [--stdout|--diff] [--check]
envq set KEY - PATH [--stdout|--diff] [--check]
envq clear KEY PATH [--stdout|--diff] [--check]
envq unset KEY PATH [--stdout|--diff] [--check]
envq remove KEY PATH [--stdout|--diff] [--check]
envq has KEY PATH
envq list PATH [--json|--yaml|--names] [--unique]
envq completion {bash,zsh,fish,powershell,pwsh}
envq help [COMMAND]
```

Examples:

```bash
printf 'A=1\nB=two\n' > /tmp/envq-demo.env

envq get A /tmp/envq-demo.env
envq list /tmp/envq-demo.env
envq list /tmp/envq-demo.env --json

envq set A 2 /tmp/envq-demo.env --diff
envq set A 2 /tmp/envq-demo.env --stdout
envq set A 2 /tmp/envq-demo.env

printf 'line1\nline2\n' | envq set SECRET - /tmp/envq-demo.env --stdout
```

## Commands

- `get KEY PATH` prints the first matching value exactly, without adding a
  newline.
- `has KEY PATH` prints nothing and reports presence through the exit code.
- `list PATH` prints bindings in file order as `KEY<TAB>VALUE`.
- `list --json`, `list --yaml`, and `list --names` change list output format.
- `list --unique` keeps the first binding for each key.
- `set KEY VALUE PATH` updates the first matching binding, appends a new
  binding, or creates `PATH` when the file is missing and the parent directory
  exists.
- `set KEY - PATH` reads the value from stdin exactly, including trailing
  newlines.
- `clear KEY PATH` is equivalent to `set KEY "" PATH`.
- `unset KEY PATH` removes all matching bindings and exits `2` if the key is
  absent.
- `remove KEY PATH` is an alias for `unset`.

Mutating commands write by default. Trailing `--stdout` prints the rendered file
without writing, trailing `--diff` prints a unified diff without writing, and
trailing `--check` never writes and exits with code `4` when the file would
change. `--check` may be combined with `--stdout` or `--diff`.

Output options are parsed only after normal operands, so this stores `--stdout`
as the value:

```bash
envq set KEY --stdout .env
```

## Exit Codes

| Code | Meaning |
| --- | --- |
| 0 | Success |
| 1 | General error |
| 2 | Key not found |
| 3 | Validation error |
| 4 | Would change |

`--quiet` suppresses non-success diagnostics on stderr while preserving exit
codes. Usage errors print usage to stderr and exit `1`.

`completion` supports bash, zsh, fish, PowerShell, and pwsh. `cmd.exe`
completions are not supported.

## Syntax

Supported bindings:

```env
KEY=value
export KEY=value
KEY=
KEY="value"
KEY='value'
```

Keys must match `[A-Za-z_][A-Za-z0-9_]*`. Blank lines, full-line comments, and
malformed or unsupported lines are preserved.

Inline comments are recognized only when `#` is preceded by horizontal
whitespace in an unquoted value:

```env
KEY=value # comment
KEY=value#not-comment
KEY=#not-comment
```

Unsupported syntax is preserved as invalid text, not interpreted. This includes
shell execution, variable expansion, multiline physical values, `KEY: value`,
unassigned names, quoted keys, and broad dotenv compatibility extensions.

## Quoting

Unquoted values are read literally. Single-quoted values strip the surrounding
quotes and otherwise read contents literally. Double-quoted values decode `\\`,
`\"`, `\n`, `\r`, and `\t`; unknown escape sequences remain literal, including
the backslash.

When writing, safe values remain unquoted. Values containing whitespace, `#`,
quotes, backslash, or control characters are double-quoted. Double-quoted output
escapes backslash, quote, newline, carriage return, and tab.

## Duplicates, Newlines, And Rewrites

Duplicate keys are allowed. `get` returns the first match, `has` succeeds if any
match exists, `list` includes duplicates in file order, `set` and `clear` update
the first match only, and `unset`/`remove` delete all matches.

Untouched lines preserve their text and line terminator. Changed lines retain
their existing terminator. Appended lines and required terminal newlines use the
first newline style found in the file. When no newline style exists, including a
newly created file, envq uses the platform default: LF on POSIX systems and CRLF
on Windows.

If a mutating command removes every line, the resulting file is empty.
Otherwise, rewritten files end with a newline.

Mutating commands write through a temporary file in the same directory and
atomically replace the target path. Atomic replacement targets the path itself:
if the path is a symlink, the symlink is replaced by a regular file and the old
symlink target is left untouched. Existing file mode is preserved where the
platform supports POSIX mode bits. envq does not take file locks; concurrent
writers race with last-replace-wins filesystem semantics.

## Development

Run the main local checks:

```bash
make ci
```

Before tagging a release, run:

```bash
make pre-release
```

Useful individual checks:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked
cargo +1.88.0 test --locked
cargo package --locked --list --allow-dirty
```

Local fuzzing is available through `cargo-fuzz`:

```bash
make fuzz-build
make fuzz-smoke
```

## License

BSD-3-Clause. See [LICENSE](https://github.com/techouse/envq/blob/main/LICENSE).
