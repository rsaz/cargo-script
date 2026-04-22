# cargo-run examples

Each subdirectory is a self-contained, runnable example showcasing one
feature of cargo-run. From the root of any subdirectory, run:

```bash
cargo script <task>
```

| Directory                    | Demonstrates                                  |
| ---------------------------- | --------------------------------------------- |
| `workspace/`                 | Multi-crate workspace, parallel test runs     |
| `cargo-scripts/`             | `.rs` script files declared in `Scripts.toml` |
| `ci-cd/`                     | Templates and machine-readable JSON output    |
| `advanced/`                  | Hooks, parameters, watch mode                 |

These examples are intentionally tiny so they read in one screenful.
