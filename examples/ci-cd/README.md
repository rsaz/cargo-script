# CI/CD example

`Scripts.toml` + GitHub Actions workflow that uses cargo-run as the single
entry point. The workflow:

1. Installs cargo-run.
2. Runs `cargo script ci --json | tee result.json`.
3. Asserts the JSON `success` flag with `jq -e`.

```bash
cd examples/ci-cd
cargo script ci             # local dry-run
cargo script ci --json      # machine-readable output
cargo script ci --dry-run   # see what would run
```

Want a different CI provider? Use a template:

```bash
cargo script init --template gitlab-ci
```
