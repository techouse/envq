## Description

Describe the change, why it is needed, and the behavior it affects.

Fixes #(issue)

## Type of Change

Delete options that are not relevant.

- [ ] Bug fix
- [ ] New command or behavior
- [ ] Breaking behavior change
- [ ] Documentation or fixture update
- [ ] CI or packaging change

## Compatibility

- [ ] README behavior contract still matches the implementation
- [ ] Golden fixtures were added or updated for behavior changes
- [ ] Invalid UTF-8 and raw byte behavior were considered

## Testing

List the commands you ran.

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked
```

## Checklist

- [ ] I have performed a self-review
- [ ] I have kept the change scoped to envq behavior
- [ ] New and existing tests pass locally
- [ ] Documentation and fixtures were updated where needed
