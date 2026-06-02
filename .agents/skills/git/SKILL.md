---
name: git
description: Git commit guidelines using conventional commits. Use when creating commits, writing commit messages.
---

# Git Commit

## Conventional Commits Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Commit Types

- `feat`: New features (correlates with MINOR in semantic versioning)
- `fix`: Bug fixes (correlates with PATCH in semantic versioning)
- `docs`: Documentation only changes
- `refactor`: Code changes that neither fix bugs nor add features
- `perf`: Performance improvements
- `test`: Adding or modifying tests
- `chore`: Maintenance tasks, dependency updates, etc.
- `style`: Code style changes (formatting, missing semicolons, etc.)
- `build`: Changes to build system or dependencies
- `ci`: Changes to CI configuration files and scripts

### Scope Guidelines

- **Scope is OPTIONAL**: only add when it provides clarity
- Use lowercase, placed in parentheses after type: `feat(transcription):`
- Prefer specific component/module names over generic terms
- Your current practice is good: component names (`EditRecordingDialog`), feature areas (`transcription`, `sound`)
- Avoid overly generic scopes like `ui` or `backend` unless truly appropriate

### When to Use Scope

- When the change is localized to a specific component/module
- When it helps distinguish between similar changes
- When working in a large codebase with distinct areas

### When NOT to Use Scope

- When the change affects multiple areas equally
- When the type alone is sufficiently descriptive
- For small, obvious changes

### Description Rules

- Start with lowercase immediately after the colon and space
- Use imperative mood ("add" not "added" or "adds")
- No period at the end
- Keep under 50-72 characters on first line

### Breaking Changes

- Add `!` after type/scope, before colon: `feat(api)!: change endpoint structure`
- Include `BREAKING CHANGE:` in the footer with details
- These trigger MAJOR version bumps in semantic versioning

## Commit Messages Best Practices

### The "Why" is More Important Than the "What"

The commit message subject line describes WHAT changed. The commit body explains WHY.

**Good commit** (explains motivation):

```
fix(auth): prevent session timeout during file upload

Users were getting logged out mid-upload on large files because the
session refresh only triggered on navigation, not background activity.
```

**Bad commit** (only describes what):

```
fix(auth): add keepalive call to upload handler
```

The first commit tells future developers WHY the code exists. The second makes them dig through the code to understand the purpose.

### Other Best Practices

- NEVER include Github copilot, Claude Code or opencode watermarks or attribution
- Each commit should represent a single, atomic change
- Write commits for future developers (including yourself)
- If you need more than one line to describe what you did, consider splitting the commit
