# Contributing to LAST Stack

First off — thank you for taking the time to contribute! 🦀  
Every bug report, suggestion, and pull request makes this project better for everyone.

This document outlines how to contribute effectively and respectfully.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Features](#suggesting-features)
- [Your First Pull Request](#your-first-pull-request)
- [Development Setup](#development-setup)
- [Commit Convention](#commit-convention)
- [Code Style](#code-style)
- [Pull Request Checklist](#pull-request-checklist)
- [Questions & Discussions](#questions--discussions)

---

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/).  
By participating, you agree to uphold a welcoming and respectful environment for everyone — regardless of experience level, background, or identity.

**In short:** be kind, be constructive, assume good intent.

---

## How Can I Contribute?

You don't have to write code to contribute. There are many ways to help:

- **Report a bug** — something doesn't work as expected
- **Suggest a feature** — an idea that would improve the project
- **Improve documentation** — fix a typo, clarify an explanation, add an example
- **Review a pull request** — share feedback on open PRs
- **Write tests** — improve coverage for an existing service
- **Write code** — fix a bug or implement a feature

---

## Reporting Bugs

Before opening a bug report, please search [existing issues](../../issues) to avoid duplicates.

When filing a bug, include as much detail as possible:

- **Summary** — a clear, one-sentence description of the problem
- **Steps to reproduce** — the exact sequence of actions that triggers the bug
- **Expected behaviour** — what you expected to happen
- **Actual behaviour** — what actually happened
- **Environment** — OS, Rust version (`rustc --version`), Docker version
- **Logs** — relevant output from `cargo run` or `docker compose logs`

Use the **Bug Report** issue template when available.

---

## Suggesting Features

Feature requests are very welcome. Before opening one, consider:

- Does this fit the scope of a Rust fullstack example project?
- Is there an existing issue or discussion about it?

Open a **Feature Request** issue and describe:

- The problem you are trying to solve
- Your proposed solution
- Any alternatives you considered
- Whether you are willing to implement it yourself

A maintainer will label it and respond as soon as possible.

---

## Your First Pull Request

Not sure where to start? Look for issues labelled:

- `good first issue` — well-scoped, beginner-friendly tasks
- `help wanted` — tasks where contributions are especially welcome
- `documentation` — no Rust experience required

### Workflow

1. **Fork** the repository and clone your fork locally.

2. **Create a branch** from `main`:
   ```bash
   git checkout -b feat/my-feature
   # or
   git checkout -b fix/issue-42
   ```

3. **Make your changes.** Keep commits small and focused — one logical change per commit.

4. **Run checks locally** before pushing (see [Pull Request Checklist](#pull-request-checklist)).

5. **Push** your branch and open a pull request against `main`.

6. **Fill out the PR template** — describe what changed and why.

7. **Respond to review feedback.** Maintainers may request changes; this is normal and not a rejection.

PRs are merged by a maintainer once they pass CI and receive at least one approval.

---

## Development Setup

The fastest way to get started is with the included Dev Container. See the [README](README.md#getting-started) for full instructions.

### Manual setup (without Dev Container)

```bash
# Rust stable + WASM target
rustup update stable
rustup target add wasm32-unknown-unknown

# Leptos build tool
cargo install trunk

# Live reloading
cargo install cargo-watch

# Start infrastructure
docker compose up zookeeper kafka surrealdb -d
```

### Running a single service

```bash
cargo watch -x "run --bin todo-service"
```

### Running all tests

```bash
cargo test --workspace
```

---

## Commit Convention

This project follows [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

```
<type>(<scope>): <short summary>
```

| Type | When to use |
|------|-------------|
| `feat` | A new feature |
| `fix` | A bug fix |
| `docs` | Documentation only |
| `refactor` | Code change that is neither a fix nor a feature |
| `test` | Adding or updating tests |
| `chore` | Tooling, CI, dependency updates |
| `perf` | Performance improvement |

**Examples:**

```
feat(todo-service): add pagination to list endpoint
fix(kafka): handle reconnect on broker timeout
docs(readme): clarify dev container setup steps
chore(deps): update axum to 0.7.5
```

- Use the **imperative mood** in the summary: "add", not "added" or "adds"
- Keep the summary under **72 characters**
- Reference issues with `Closes #42` or `Fixes #17` in the commit body when applicable

---

## Code Style

### Formatting

All Rust code must be formatted with `rustfmt` before committing:

```bash
cargo fmt --all
```

CI will reject unformatted code.

### Linting

Run Clippy and fix all warnings:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

New code should introduce zero Clippy warnings. If suppressing a lint with `#[allow(...)]` is genuinely necessary, add a comment explaining why.

### General guidelines

- Prefer explicit error handling over `.unwrap()` — use `?` and proper error types from `shared::errors`
- Write `async` code with Tokio; avoid blocking calls on async threads
- Keep service boundaries clear — a service should not import another service's crate directly, only `shared`
- Add a doc comment (`///`) to every public function, struct, and trait
- Tests live in a `#[cfg(test)]` module at the bottom of the file they test, or in `tests/` for integration tests

---

## Pull Request Checklist

Before marking a PR as ready for review, confirm that all of the following are true:

- [ ] My branch is up to date with `main`
- [ ] `cargo fmt --all` produces no diff
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] New behaviour is covered by at least one test
- [ ] Public APIs and non-obvious logic have doc comments
- [ ] The PR description explains *what* changed and *why*
- [ ] Related issues are referenced (`Closes #...`)

---

## Questions & Discussions

For questions that are not bug reports or feature requests, please use [GitHub Discussions](../../discussions) rather than opening an issue. This keeps the issue tracker focused on actionable tasks.

You can also reach out via the Discussions board to:

- Ask about architecture decisions
- Share what you built on top of LAST Stack
- Propose larger changes before writing code (recommended for significant refactors)

---

Thank you for contributing — every bit helps. Happy hacking! 🦀
