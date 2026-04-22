# Advanced features example

Showcases hooks, parameter substitution, and watch mode.

```bash
cd examples/advanced

# Parameterised deploy with hooks
cargo script deploy                  # uses default env=staging
cargo script deploy production       # positional
cargo script deploy env=qa           # named

# Watch a script — re-runs on file changes
cargo script test-watch --watch
cargo script test-watch --watch --watch-path src --watch-exclude target
```

See [`docs/ADVANCED_FEATURES.md`](../../docs/ADVANCED_FEATURES.md) for the
full reference.
