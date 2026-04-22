# Community Outreach Plan — `cargo-run` 0.6

This document is the operational checklist for the v0.6 launch. It covers the
three concrete asks from the strategic plan:

1. **Submit to *This Week in Rust*** (PR-driven).
2. **Post on `/r/rust`** (and adjacent communities).
3. **Engage with the RFC 3502 (`cargo-script`) tracking issue.**

Every section contains ready-to-paste copy. Replace `<RELEASE-DATE>` and
`<PR-LINK>` placeholders before publishing.

---

## 1. *This Week in Rust* submission

### Where

- Repo: <https://github.com/rust-lang/this-week-in-rust>
- File: `content/<YYYY-MM-DD>-this-week-in-rust.md` (the latest draft)
- Section: **"Updates from the Rust Project"** is reserved — use **"Crate of
  the Week"** *(only if explicitly nominated)* or, for a release announcement,
  **"Newsletters"** → **"Project/Tooling Updates"**.

The de-facto convention is to add a single bullet under
**"Project/Tooling Updates"**.

### PR title

```
Add cargo-run 0.6 release announcement
```

### PR body

```
This adds a one-line entry under "Project/Tooling Updates" announcing the
0.6 release of cargo-run, a workspace-aware task runner for Rust with
first-class support for RFC 3502 (cargo-script).

Closes the editor's nightly review checklist if anything else is needed.
```

### Bullet to add

```markdown
* [cargo-run 0.6](https://github.com/lpotthast/cargo-run) — workspace
  orchestration, first-class `cargo-script` (RFC 3502) support, CI/CD
  templates, lifecycle hooks, parallel execution, watch mode, parameter
  substitution and JSON output.
```

### Etiquette checklist

- [ ] Open the PR on a Tuesday or Wednesday (TWiR cadence).
- [ ] Use a descriptive PR title; reviewers occasionally rephrase the bullet.
- [ ] Be responsive to editor comments within 24h.
- [ ] If nominated for **Crate of the Week**, prepare a 2-sentence pitch
      (see template below).
- [ ] Don't self-nominate as Crate of the Week the same week as the release
      — submit the announcement first; let community members nominate.

### Crate-of-the-Week pitch (if asked)

> `cargo-run` is a workspace-aware task runner for Rust that bridges
> `Scripts.toml` and the upcoming stable `cargo-script` (RFC 3502). One
> declarative file replaces a Justfile, a Makefile, an `xtask` crate and
> half a dozen shell scripts — with built-in parallel execution, watch
> mode, hooks, parameters, CI/CD templates and JSON output.

---

## 2. `/r/rust` post

### Where

- Subreddit: <https://www.reddit.com/r/rust/>
- Allowed flair: **"🛠️ project"** (or **"📢 announcement"** if available).
- Time: weekday morning **US Eastern** (highest engagement).

### Title

```
cargo-run 0.6 — workspace-aware task runner for Rust, with first-class cargo-script (RFC 3502) support
```

### Body (Markdown)

```markdown
Hi r/rust,

I just released **cargo-run 0.6**, a task runner for Rust projects. The
0.6 release is a major one — it grows from "small Scripts.toml runner" into
a workspace-aware tool that bridges today's Cargo and the upcoming stable
`cargo-script` (RFC 3502).

**Highlights**

- 🧩 **Workspace orchestration** — `workspace = "all" | "parallel" | "root"`
  per script. No more `xtask` crates or `for` loops.
- 📜 **Cargo-script (RFC 3502) integration** — declare `.rs` files in
  `Scripts.toml`; auto-detects stable/nightly toolchains.
- 🪄 **CI/CD templates** — `cargo script init --template github-actions`
  scaffolds `Scripts.toml` + the matching workflow.
- 🪝 **Lifecycle hooks** — `pre`, `post`, `on_success`, `on_failure`.
- ⚡ **Parallel execution** — Tokio-powered, gated behind a feature flag.
- 👀 **Watch mode** — `--watch`, `--watch-path`, `--watch-exclude`.
- 🧮 **Parameters** — `args`, `defaults`, `{{name}}` substitution.
- 📊 **JSON output** — `--json` for pipeline-friendly results.

**A taste**

```toml
[scripts.test]
command   = "cargo test"
workspace = "parallel"

[scripts.audit]
script_file = "./scripts/audit.rs"   # RFC 3502 .rs script

[scripts.deploy]
command  = "kubectl apply -f manifests/{{env}}.yaml"
args     = ["env"]
defaults = { env = "staging" }
pre      = ["fmt-check", "lint", "test"]
on_failure = ["rollback"]
```

```bash
cargo install cargo-run --features watch,parallel
cargo script init --template rust-project
cargo script test --watch --json
```

**Links**

- Repo: https://github.com/lpotthast/cargo-run
- Announcement post: https://github.com/lpotthast/cargo-run/blob/main/docs/blog/01-announcing-cargo-run-0.6.md
- Migration guides (just / make / cargo-make / npm scripts):
  https://github.com/lpotthast/cargo-run/blob/main/docs/MIGRATION_GUIDES.md

Happy to answer questions in the thread — particularly interested in
feedback from anyone running multi-crate workspaces or already
experimenting with `cargo-script` on nightly.
```

### Etiquette checklist

- [ ] Read [the r/rust posting rules](https://www.reddit.com/r/rust/wiki/rules)
      before posting; cross-posting from a personal blog is fine, drive-by
      promotion isn't.
- [ ] Reply to early comments within the first hour — this is what the algorithm
      and the moderators look for.
- [ ] Disclose maintainer status in the post (e.g. *"I maintain it"*).
- [ ] No follow-up self-promo posts for at least two weeks.

### Adjacent communities

- [ ] **Hacker News** — submit the announcement post (not the README) with the
      title `Show HN: cargo-run 0.6 – workspace-aware task runner for Rust`.
- [ ] **Lobste.rs** — tag `rust`, `release`. Submit the announcement post.
- [ ] **Rust Users Forum** — `#announcements` category.
- [ ] **dev.to** / **Hashnode** — re-publish post 1 with `canonical_url`
      pointing back to the GitHub repo.

---

## 3. RFC 3502 tracking-issue engagement

### Where

- RFC: <https://rust-lang.github.io/rfcs/3502-cargo-script.html>
- Tracking issue: <https://github.com/rust-lang/cargo/issues/12207>
  *(verify the latest tracking issue at release time)*

### Goal

Position `cargo-run` as a **complementary tool**, not a competitor:
`cargo-script` is the *engine*; `cargo-run` is the *registry/orchestrator*
that wires those scripts into project workflow.

### Comment template

```markdown
We've shipped first-class support for RFC 3502 in
[cargo-run 0.6](https://github.com/lpotthast/cargo-run) (a task runner for
Rust). Users can declare a `.rs` file as a project task:

```toml
[scripts.audit]
script_file = "./scripts/audit.rs"
info        = "Audit dependencies (Rust script, RFC 3502)"
```

cargo-run auto-detects whether stable `cargo script` or `cargo +nightly
-Zscript` is available and falls back gracefully, so the same
`Scripts.toml` works across toolchains during the stabilisation period.

A few pieces of feedback from integrating against the current nightly:

1. **Front-matter discoverability** — having a programmatic way to query
   "what cargo-script frontmatter does this file have?" would let
   tooling (us, clippy, IDEs) avoid re-parsing the `---` block ourselves.
2. **`cargo script --json`** — a stable JSON output for cargo-script
   invocations would make it much easier for orchestrators like cargo-run
   to attribute compile time vs. runtime.
3. **Cache location** — exposing the cached binary path (or stable env var)
   would let task runners snapshot artefacts in CI without re-compilation.

Happy to open separate issues for any of these if useful. Big thanks to
the cargo team — RFC 3502 unlocks a class of tooling we've been waiting
years for.
```

### Etiquette checklist

- [ ] Comment **once**, with substance. Tracking issues are not promotion
      surfaces.
- [ ] Lead with concrete feedback or a use-case, not the announcement.
- [ ] Open separate issues for anything actionable so the tracking issue
      stays scannable.
- [ ] If a maintainer asks for help, prioritise it.

---

## Timeline

| Day | Action                                                   |
| --- | -------------------------------------------------------- |
| -3  | Cut the v0.6 release on crates.io, draft GH release.     |
| -1  | Final dry-run of the four blog posts.                    |
| 0   | **Publish announcement** (post 1) on `/r/rust` + HN + Lobste.rs. |
| 0   | Open the **TWiR PR**.                                    |
| +2  | Publish post 2 (workspace deep dive) on dev.to.          |
| +5  | Publish post 3 (cargo-script integration). Comment on RFC 3502 tracking issue. |
| +10 | Publish post 4 (migration guide). Cross-post to dev.to + Hashnode. |
| +14 | Round-up: respond to issues, schedule v0.6.x bug-fix release if needed. |

## Tracking

Use a private spreadsheet (or GitHub project board) with columns:

- Channel | URL | Date posted | Upvotes / reactions | Comments handled? | Notes

This gives us data for the next release.

---

*Generated as part of the cargo-run 0.6 evolution plan. Update this file as
the launch progresses.*
