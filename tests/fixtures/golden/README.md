# Golden Fixture Schema

Golden fixtures are the portable behaviour oracle for envq. They are JSON so
they can be consumed by simple fixture loaders in any implementation.

Each manifest has this shape:

```json
{
  "schema_version": 1,
  "cases": []
}
```

Every case has:

- `id`: globally unique stable identifier.
- `kind`: `parse`, `edit`, or `cli`.
- `platform`: `all`, `posix`, or `windows`.
- `input`: either inline text or a sidecar file reference.
- `expect`: expected result object for the case kind.

Text or bytes objects use one of these forms:

```json
{ "text": "A=1\n" }
{ "file": "files/input.env", "format": "utf-8" }
{ "file": "files/input.escaped", "format": "escaped-bytes" }
{ "same_as_input": true }
{ "missing": true }
{ "missing_parent": true }
{ "directory": true }
```

`escaped-bytes` sidecars encode bytes with `\n`, `\r`, `\t`, `\\`, and `\xNN`. This keeps CRLF, mixed-newline, and invalid-UTF-8 fixtures reviewable in Git.

Parse cases require `expect.bindings`, `expect.line_kinds`, and `expect.rendered`. `rendered` may use `{ "same_as_input": true }`.

Edit cases require `operation`, `expect.output`, and, for `unset`, `expect.removed_count`. Supported operations are `set`, `clear`, and `unset`.

CLI cases require `args`, `expect.exit_code`, `expect.stdout`, `expect.stderr`, and `expect.file`. `args`, `stdout`, and `stderr` string values may contain `{path}`, which the test harness expands to the temporary fixture path, or `{version}`, which expands to the Cargo package version. `stdout` and `stderr` may also use any text or bytes object except `same_as_input`. CLI `input` may be `{ "missing": true }`, `{ "missing_parent": true }`, or `{ "directory": true }` for file-error cases.
