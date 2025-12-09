# Implementation Plan: Golden Standard Features

## Overview
This document outlines the step-by-step implementation plan for adding golden standard CLI features to `cargo-run`. We'll implement one feature at a time, test it thoroughly, then move to the next.

---

## Feature Priority Order

### ✅ Feature #1: Shell Autocompletion (HIGH PRIORITY)
**Impact:** High | **Effort:** Low | **Time:** 2-3 hours

**Why First:**
- Critical for developer experience
- Easy to implement with clap's built-in support
- Immediate user value
- Low risk

**Implementation Steps:**
1. Add `clap_complete` dependency
2. Add `Completions` subcommand to Commands enum
3. Generate completions for bash, zsh, fish, PowerShell
4. Add dynamic script name completion for `run` command
5. Write tests
6. Update documentation

**Acceptance Criteria:**
- ✅ `cgs completions bash` generates bash completion script
- ✅ `cgs completions zsh` generates zsh completion script
- ✅ `cgs completions fish` generates fish completion script
- ✅ `cgs completions powershell` generates PowerShell completion script
- ✅ Tab completion works for commands (run, init, show, completions)
- ✅ Tab completion suggests script names from Scripts.toml
- ✅ Tests verify completion generation works

**Testing:**
- Unit tests for completion generation
- Integration test for script name completion
- Manual verification on each shell

---

### ✅ Feature #2: Dry-Run Mode
**Impact:** High | **Effort:** Low | **Time:** 2-3 hours

**Why Second:**
- Users want to preview before execution
- Prevents accidental runs
- Standard in modern CLI tools

**Implementation Steps:**
1. Add `--dry-run` flag to `Run` command
2. Parse and validate scripts without executing
3. Display execution plan (what would run)
4. Show environment variables that would be set
5. Write tests
6. Update documentation

**Acceptance Criteria:**
- ✅ `cgs run <script> --dry-run` shows execution plan
- ✅ Shows script chain (includes)
- ✅ Shows environment variables
- ✅ Shows commands that would execute
- ✅ Does NOT execute anything
- ✅ Tests verify dry-run output

**Testing:**
- Test dry-run shows correct plan
- Test dry-run doesn't execute
- Test dry-run with script chains
- Test dry-run with environment variables

---

### ✅ Feature #3: Better Error Messages
**Impact:** High | **Effort:** Medium | **Time:** 3-4 hours

**Why Third:**
- Improves user experience significantly
- Reduces support burden
- Makes tool more professional

**Implementation Steps:**
1. Create custom error types
2. Add context to error messages
3. Add suggestions (e.g., "Did you mean...?")
4. Add links to documentation
5. Improve Scripts.toml parsing errors
6. Write tests

**Acceptance Criteria:**
- ✅ Script not found suggests similar names
- ✅ Invalid TOML shows line numbers and context
- ✅ Missing tool shows installation instructions
- ✅ Errors include actionable suggestions
- ✅ Tests verify error messages

**Testing:**
- Test script not found error
- Test invalid TOML error
- Test missing tool error
- Test error message formatting

---

### ✅ Feature #4: Validation Command
**Impact:** Medium | **Effort:** Low | **Time:** 2 hours

**Why Fourth:**
- Helps catch errors early
- Useful in CI/CD
- Standard feature

**Implementation Steps:**
1. Add `Validate` subcommand
2. Parse Scripts.toml
3. Validate script references (includes)
4. Check tool requirements
5. Report all issues
6. Write tests

**Acceptance Criteria:**
- ✅ `cgs validate` checks Scripts.toml syntax
- ✅ Validates script references in includes
- ✅ Checks tool requirements
- ✅ Reports all issues, not just first
- ✅ Exit code 0 on success, non-zero on failure
- ✅ Tests verify validation

**Testing:**
- Test valid Scripts.toml
- Test invalid syntax
- Test missing script references
- Test missing tools

---

### ✅ Feature #5: Cargo Subcommand Registration
**Impact:** High | **Effort:** Medium | **Time:** 4-5 hours

**Why Fifth:**
- Critical for Rust ecosystem adoption
- Users expect `cargo script` not just `cargo-script`
- Requires careful implementation

**Implementation Steps:**
1. Research Cargo subcommand protocol
2. Create `cargo-script` binary that Cargo can invoke
3. Handle `cargo script` vs `cargo-script` distinction
4. Update installation instructions
5. Write tests
6. Update documentation

**Acceptance Criteria:**
- ✅ `cargo script run dev` works
- ✅ `cargo script init` works
- ✅ `cargo script show` works
- ✅ Backward compatible with `cgs` and `cargo-script`
- ✅ Tests verify subcommand works

**Testing:**
- Test cargo subcommand invocation
- Test backward compatibility
- Test all commands work as subcommand

---

## Implementation Workflow

For each feature:

1. **Plan** - Review this document and feature details
2. **Implement** - Write code following existing patterns
3. **Test** - Write comprehensive tests
4. **Verify** - Run tests and manual testing
5. **Document** - Update README and docs
6. **Commit** - Commit with clear message
7. **Review** - Get feedback before next feature

---

## Testing Strategy

### Unit Tests
- Test individual functions
- Mock external dependencies
- Fast execution

### Integration Tests
- Test full command execution
- Use `assert_cmd` crate
- Test real Scripts.toml files

### Manual Testing
- Test on different shells (bash, zsh, fish, PowerShell)
- Test on different platforms (Windows, Linux, macOS)
- Test edge cases

---

## Code Quality Standards

- Follow existing code style
- Add documentation comments
- Handle errors gracefully
- Use meaningful variable names
- Keep functions focused and small
- Write tests for new code

---

## Progress Tracking

- [ ] Feature #1: Shell Autocompletion
- [ ] Feature #2: Dry-Run Mode
- [ ] Feature #3: Better Error Messages
- [ ] Feature #4: Validation Command
- [ ] Feature #5: Cargo Subcommand Registration

---

## Notes

- Each feature should be fully tested before moving to next
- Keep commits atomic (one feature per commit)
- Update this document as we progress
- Get user feedback after each feature

