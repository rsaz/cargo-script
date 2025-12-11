# Cargo-Script Roadmap

This document outlines potential future improvements and features for the cargo-script project.

## High-Impact Features (From Original Roadmap)

### 1. Watch Mode ⭐
Auto-rerun scripts when files change (similar to `cargo watch`).

**Usage:**
```bash
cargo script test --watch
cargo script build --watch --watch-path src/
cargo script lint --watch --watch-exclude target/
```

**Use Cases:**
- Auto-testing during development
- Auto-rebuilding on file changes
- Auto-formatting/linting

**Implementation Notes:**
- Use `notify` crate for file watching
- Support `--watch-path` to specify directories to watch
- Support `--watch-exclude` to exclude paths
- Show which files changed before re-running

---

### 2. Parallel Execution ⭐
Run independent scripts concurrently for better performance.

**Usage:**
```toml
[scripts]
build-all = {
    include = ["build-rust", "build-python", "build-docs"],
    parallel = true  # Run includes in parallel
}
```

**Implementation Notes:**
- Detect independent scripts (no shared dependencies)
- Use `tokio` or `rayon` for parallel execution
- Show parallel execution status
- Handle errors from parallel executions gracefully

---

### 3. Conditional Execution ⭐
Add `if`/`unless` conditions to scripts.

**Usage:**
```toml
[scripts]
deploy = {
    command = "./deploy.sh",
    if = { env = "CI" },  # Only run in CI
    unless = { file_exists = "maintenance.txt" }
}

test-integration = {
    command = "cargo test --test integration",
    if = { env_var = "RUN_INTEGRATION_TESTS" }
}
```

**Implementation Notes:**
- Support environment variable checks
- Support file existence checks
- Support command exit code checks
- Evaluate conditions before script execution

---

### 4. Script Templates ⭐
Pre-built workflow templates for common scenarios.

**Usage:**
```bash
cargo script init --template ci
cargo script init --template rust-project
cargo script init --template full-stack
```

**Templates:**
- `ci` - CI/CD pipeline scripts
- `rust-project` - Standard Rust project scripts
- `full-stack` - Full-stack application scripts
- `library` - Library project scripts

---

## Medium-Impact Improvements

### 5. Script Aliases/Shortcuts
Short aliases for common scripts.

**Usage:**
```toml
[scripts]
b = "build"  # Alias
t = "test"
d = "dev"
```

**Implementation:**
- Support string aliases in Scripts.toml
- Resolve aliases before execution
- Show aliases in `cargo script show`

---

### 6. Script Dependencies (depends_on)
Ensure scripts run in the correct order automatically.

**Usage:**
```toml
[scripts]
deploy = {
    command = "./deploy.sh",
    depends_on = ["build", "test"]  # Auto-run dependencies first
}
```

**Implementation:**
- Build dependency graph
- Detect circular dependencies
- Run dependencies in correct order
- Skip if dependency already ran

---

### 7. Script Variables/Parameters
Pass arguments to scripts dynamically.

**Usage:**
```bash
cargo script deploy staging
cargo script deploy production
cargo script build --target x86_64-unknown-linux-gnu
```

**Script Definition:**
```toml
[scripts]
deploy = {
    command = "./deploy.sh {{env}}",
    args = ["env"]  # Accepts environment argument
}

build = {
    command = "cargo build --target {{target}}",
    args = { target = "x86_64-unknown-linux-gnu" }  # With defaults
}
```

---

### 8. Script Groups/Categories
Organize scripts into logical groups.

**Usage:**
```toml
[scripts.build]
release = "cargo build --release"
debug = "cargo build"

[scripts.test]
unit = "cargo test"
integration = "cargo test --test integration"

[scripts.deploy]
staging = "./deploy.sh staging"
production = "./deploy.sh production"
```

**CLI:**
```bash
cargo script build:release
cargo script test:unit
cargo script show --group build
```

---

### 9. Script Execution History/Logging
Track script runs for debugging and analytics.

**Usage:**
```bash
cargo script history
cargo script history test --last 10
cargo script history --failed
cargo script history --since "2024-01-01"
```

**Features:**
- Log execution time, exit codes, errors
- Store in `.cargo-script/history.json`
- Show statistics (most run scripts, average time, etc.)

---

### 10. Script Timeouts
Set timeouts for long-running scripts.

**Usage:**
```toml
[scripts]
long-task = {
    command = "./long-running.sh",
    timeout = "5m"  # 5 minutes
}

quick-check = {
    command = "./check.sh",
    timeout = "30s"
}
```

**Implementation:**
- Parse timeout strings (e.g., "5m", "30s", "1h")
- Kill process if timeout exceeded
- Show timeout warning before execution

---

## Developer Experience Improvements

### 11. Better Script Discovery
Enhanced search and discovery features.

**Commands:**
```bash
cargo script search <pattern>        # Search scripts by name/description
cargo script list --tree             # Show script dependencies as tree
cargo script info <script>           # Detailed info about a script
cargo script show --sort-by usage    # Sort by usage frequency
```

**Features:**
- Tree view of script dependencies
- Search across names and descriptions
- Show script metadata (last run, success rate, etc.)

---

### 12. Script Documentation
Add markdown documentation to scripts.

**Usage:**
```toml
[scripts]
deploy = {
    command = "./deploy.sh",
    docs = """
    Deploys the application to production.
    
    Requirements:
    - AWS credentials configured
    - Docker installed
    
    Examples:
    - cargo script deploy staging
    - cargo script deploy production --dry-run
    """
}
```

**CLI:**
```bash
cargo script docs deploy
cargo script show --with-docs
```

---

### 13. Script Validation Improvements
Enhanced validation with more checks.

**Checks:**
- Detect circular dependencies in `include`
- Warn about unused scripts
- Check for duplicate script names
- Validate script arguments
- Check for conflicting aliases
- Warn about scripts that never run

**Usage:**
```bash
cargo script validate --strict
cargo script validate --check-unused
```

---

### 14. Configuration File Improvements
More flexible configuration options.

**Features:**
- Support multiple `Scripts.toml` files (e.g., `Scripts.local.toml`)
- Import/include other config files
- Environment-specific configs (`Scripts.dev.toml`, `Scripts.prod.toml`)
- Merge configs from multiple sources

**Usage:**
```bash
cargo script --config Scripts.local.toml run build
cargo script --env dev run deploy  # Uses Scripts.dev.toml
```

---

### 15. Better Output Formatting
Enhanced output options for different use cases.

**Features:**
- JSON output mode (`--json`) for CI/CD
- Progress bars for long-running scripts
- Colored diff output for dry-run
- Machine-readable output formats
- Structured logging

**Usage:**
```bash
cargo script run build --json
cargo script run test --progress
cargo script run build --dry-run --diff
```

---

## Advanced Features

### 16. Script Hooks (pre/post)
Run scripts before/after execution.

**Usage:**
```toml
[scripts]
deploy = {
    command = "./deploy.sh",
    pre = "backup",      # Run before
    post = "notify",     # Run after (even on failure)
    on_failure = "rollback"  # Run only on failure
}
```

**Implementation:**
- Support multiple pre/post hooks
- Run hooks even if main script fails (for cleanup)
- Show hook execution in output

---

### 17. Script Retries
Retry failed scripts automatically.

**Usage:**
```toml
[scripts]
deploy = {
    command = "./deploy.sh",
    retries = 3,
    retry_delay = "5s",
    retry_on = ["network_error", "timeout"]  # Only retry on specific errors
}
```

**Features:**
- Configurable retry count
- Exponential backoff
- Retry only on specific error types
- Show retry attempts in output

---

### 18. Script Notifications
Send notifications on script completion.

**Usage:**
```toml
[scripts]
deploy = {
    command = "./deploy.sh",
    notify = {
        on = "failure",  # or "success", "always"
        webhook = "https://hooks.slack.com/...",
        email = "team@example.com"
    }
}
```

**Supported:**
- Webhooks (Slack, Discord, etc.)
- Email notifications
- Desktop notifications
- Custom notification scripts

---

### 19. Script Caching
Cache script results to avoid redundant execution.

**Usage:**
```toml
[scripts]
expensive-check = {
    command = "./expensive.sh",
    cache = {
        key = "file-hash",  # Cache key strategy
        ttl = "1h"          # Time to live
    }
}
```

**Cache Strategies:**
- File hash (cache until files change)
- Time-based (cache for TTL)
- Manual invalidation

---

### 20. Script Secrets Management
Secure handling of secrets and sensitive data.

**Usage:**
```toml
[scripts]
deploy = {
    command = "./deploy.sh",
    secrets = ["API_KEY", "SECRET_TOKEN"]  # Load from secure store
}
```

**Features:**
- Integration with secret managers (AWS Secrets Manager, HashiCorp Vault)
- Environment variable masking in output
- Secure credential storage
- Support for `.env` files with encryption

---

## Integration Improvements

### 21. IDE Integration
Better integration with development environments.

**Features:**
- VS Code extension
- IntelliSense/autocomplete for `Scripts.toml`
- Run scripts from IDE
- Show script output in IDE
- Script debugging support

---

### 22. CI/CD Improvements
Better integration with CI/CD pipelines.

**Features:**
- GitHub Actions integration
- GitLab CI templates
- Better error reporting for CI
- CI-specific script execution modes
- Parallel test execution in CI

---

### 23. Performance Improvements
Optimize for speed and efficiency.

**Improvements:**
- Faster startup time (<50ms goal)
- Parallel requirement checking
- Cached script parsing
- Lazy loading of scripts
- Optimized TOML parsing

---

## Documentation and Community

### 24. Better Documentation
Comprehensive documentation for users.

**Content:**
- Video tutorials
- More examples and use cases
- Migration guides from other tools (make, npm scripts, just)
- Best practices guide
- Troubleshooting guide
- API documentation

---

### 25. Community Features
Build a community around the tool.

**Features:**
- Script marketplace/sharing
- Common script patterns library
- Best practices guide
- Community-contributed templates
- Script sharing via GitHub Gists
- Plugin system for extensions

---

## Quick Wins (Easy to Implement, High Value)

### 1. `cargo script list` Alias
Add `list` as an alias for `show` with better formatting.

```bash
cargo script list          # Alias for show
cargo script list --tree   # Tree view
```

---

### 2. `cargo script search <pattern>`
Search scripts by name or description (enhance existing filter).

```bash
cargo script search test
cargo script search "build"
```

---

### 3. `--json` Output Mode
JSON output for CI/CD integration.

```bash
cargo script show --json
cargo script run build --json
```

---

### 4. Script Execution Count
Track how often scripts are run.

```bash
cargo script show --with-stats
cargo script stats
```

---

### 5. Better Error Context
Show which script failed in a chain.

**Current:** Shows error but not which script in the chain failed
**Improved:** Show full chain path and highlight failed script

---

## Recommended Next Steps (Priority Order)

### Phase 4: Watch Mode (High Priority)
**Why:** High developer value, commonly requested feature
**Effort:** Medium
**Impact:** High
**Dependencies:** `notify` crate

**Implementation Plan:**
1. Add `--watch` flag to Run command
2. Integrate file watcher (notify crate)
3. Support `--watch-path` and `--watch-exclude`
4. Show file change notifications
5. Add tests for watch mode

---

### Phase 5: Parallel Execution (High Priority)
**Why:** Significant performance improvement for complex workflows
**Effort:** Medium-High
**Impact:** High
**Dependencies:** `tokio` or `rayon`

**Implementation Plan:**
1. Add `parallel` flag to script configuration
2. Build dependency graph
3. Detect independent scripts
4. Implement parallel execution
5. Handle errors and output from parallel scripts
6. Add tests

---

### Phase 6: Script Parameters (Medium Priority)
**Why:** Makes scripts more flexible and reusable
**Effort:** Medium
**Impact:** Medium-High
**Dependencies:** None

**Implementation Plan:**
1. Add `args` field to Script struct
2. Parse template variables (e.g., `{{arg}}`)
3. Handle positional and named arguments
4. Update CLI to accept script arguments
5. Add validation for required arguments
6. Add tests

---

### Phase 7: Pre/Post Hooks (Medium Priority)
**Why:** Common workflow pattern, enables better script composition
**Effort:** Medium
**Impact:** Medium
**Dependencies:** None

**Implementation Plan:**
1. Add `pre`, `post`, `on_failure` fields to Script
2. Execute hooks in correct order
3. Handle hook failures gracefully
4. Show hook execution in output
5. Add tests

---

### Phase 8: JSON Output Mode (Low Priority, Quick Win)
**Why:** Better CI/CD integration, easy to implement
**Effort:** Low
**Impact:** Medium
**Dependencies:** `serde_json` (already have serde)

**Implementation Plan:**
1. Add `--json` flag
2. Create JSON output structures
3. Serialize command outputs to JSON
4. Update all commands to support JSON
5. Add tests

---

## Implementation Guidelines

### When Adding New Features:

1. **Start with tests** - Write tests first (TDD approach)
2. **Update documentation** - Keep README and docs up to date
3. **Maintain backward compatibility** - Don't break existing functionality
4. **Follow existing patterns** - Match current code style and architecture
5. **Consider performance** - Keep startup time fast
6. **Add examples** - Include usage examples in help text
7. **Update roadmap** - Mark completed features

### Code Quality Standards:

- All tests must pass
- No clippy warnings
- Code formatted with `rustfmt`
- Documentation for public APIs
- Error messages should be helpful
- Follow Rust best practices

---

## Notes

- Features marked with ⭐ are from the original roadmap
- Quick wins can be implemented in 1-2 hours each
- High-impact features may take several days to implement properly
- Consider user feedback when prioritizing features
- Some features may require breaking changes (version appropriately)

---

**Last Updated:** 2024
**Current Version:** 0.5.1

