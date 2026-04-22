# Workspace example

A two-crate workspace (`alpha`, `beta`) demonstrating workspace-aware
execution.

```bash
cd examples/workspace
cargo script test-all       # sequential
cargo script test-parallel  # parallel
cargo script ci             # full pipeline
cargo script workspace list # show discovered members
```
