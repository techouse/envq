---
name: Bug report
about: Report incorrect parsing, editing, CLI, or file I/O behavior.
title: ""
labels: bug
assignees: techouse
---

<!--
    Before filing, check the README behavior contract and the golden fixtures
    under tests/fixtures/golden/.
-->

## Problem Summary

<!--
Describe the bug clearly:
- the command or Rust API path you used
- the input file bytes
- what you expected
- what happened instead
-->

## Reproduction

Prefer a minimal command plus a tiny fixture file.

```bash
printf 'KEY=value\n' > /tmp/envq-repro.env
cargo run -- get KEY /tmp/envq-repro.env
```

## Expected Behavior

<!-- What should have happened? Include exact stdout/stderr when relevant. -->

## Actual Behavior

<!-- What happened instead? Include exact stdout/stderr and exit code. -->

## Compatibility Context

- [ ] Matches the README behavior contract
- [ ] Matches an existing golden fixture
- [ ] This may be an intentional behavior change

Relevant links or fixture names:

- Golden fixture:

## Inputs

- Command:
- File bytes or escaped bytes:
- Platform:
- Rust version:

## Environment

```bash
rustc --version
cargo --version
uname -a
```

```text
```

## Additional Context

<!-- Add logs, comparison output, or anything else that helps reproduce the issue. -->
