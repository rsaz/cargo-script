# Advanced Features

This guide covers cargo-run's power-user features:

* [Hooks](#hooks)
* [Parallel execution](#parallel-execution)
* [Watch mode](#watch-mode)
* [Script parameters](#script-parameters)
* [JSON output](#json-output)

---

## Hooks

Every inline script supports four optional lifecycle hooks:

| Hook         | Runs                                       |
| ------------ | ------------------------------------------ |
| `pre`        | Before the main command                    |
| `post`       | After the main command (always)            |
| `on_success` | After the main command, only on success    |
| `on_failure` | After the main command, only on failure    |

```toml
[scripts.deploy]
command    = "kubectl apply -f manifests/"
pre        = ["fmt-check", "lint", "test"]
post       = ["log-deploy"]
on_success = ["notify-slack-success"]
on_failure = ["rollback", "notify-slack-failure"]
```

Hooks reference other scripts by name. They can themselves be inline scripts
with their own hooks (cargo-run guards against accidental cycles by simple
recursion depth).

A failing **pre** hook short-circuits the main command. A failing **post**
hook does not (post hooks are best-effort cleanup).

---

## Parallel execution

There are two flavours of parallelism:

### 1. Workspace-parallel

Run a single script across every workspace member at the same time:

```toml
[scripts.test-parallel]
command   = "cargo test"
workspace = "parallel"
```

```bash
cargo script test-parallel
```

### 2. Multi-script parallel

Run several distinct scripts at once:

```bash
cargo script ci --parallel fmt-check --parallel lint --parallel test
```

`--parallel` may be repeated. The leading positional script name (if any) is
included in the parallel set.

Both modes:

* require the `parallel` Cargo feature (default-on).
* aggregate child results and surface a `ParallelExecutionFailed` error if
  any single child fails.
* preserve per-script output ordering inside each child but interleave output
  across children (that's the price of concurrency).

---

## Watch mode

Re-run any script on file changes:

```bash
cargo script test --watch
```

Tunable via flags:

```bash
cargo script test --watch \
    --watch-path src \
    --watch-path tests \
    --watch-exclude target \
    --watch-exclude .git
```

Defaults: watches `.`, ignores `target/`, `.git/`, `node_modules/`,
debounces events with a 250 ms window.

The watch loop:

1. Runs the script once at startup.
2. Subscribes to recursive file system events under each `--watch-path`.
3. After a debounced quiet period, re-runs the script.
4. Loops forever (Ctrl-C to exit).

Requires the default `watch` Cargo feature.

---

## Script parameters

Declare parameters with `args`, optional defaults with `defaults`, and use
`{{name}}` substitution inside `command`:

```toml
[scripts.deploy]
command  = "kubectl apply -f manifests/{{env}}.yaml"
args     = ["env"]
defaults = { env = "staging" }
```

```bash
cargo script deploy                    # uses default 'staging'
cargo script deploy production         # positional → env=production
cargo script deploy env=qa             # named
```

* Required arguments without a default raise `MissingScriptArgument`.
* Positional and `key=value` forms can mix freely.
* Substitution happens before shell expansion, so `{{x}}` is replaced
  literally — escape with quotes if the value contains spaces.

---

## JSON output

For tools that need to consume execution data, pass `--json`:

```bash
cargo script ci --json
```

Sample output:

```json
{
  "success": true,
  "script": "ci",
  "command": null,
  "duration_ms": 1843,
  "exit_code": null,
  "error": null,
  "includes": [
    {
      "success": true,
      "script": "fmt-check",
      "command": "cargo fmt --all -- --check",
      "duration_ms": 102,
      "exit_code": null,
      "error": null,
      "includes": [],
      "hooks": []
    }
  ],
  "hooks": []
}
```

Combine with `jq` for pipeline assertions:

```bash
cargo script ci --json |
  jq -e '.success and (.includes | length > 0)'
```
