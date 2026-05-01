# Git Workflow

## Branching

Branch off `main`. Never commit directly to `main`.

### Naming convention

```
<type>/<short-description>
```

| Type | When to use |
|---|---|
| `feat/` | New feature or capability |
| `fix/` | Bug fix |
| `chore/` | Tooling, deps, config — no user-facing change |
| `docs/` | Documentation only |
| `refactor/` | Code restructure, no behaviour change |
| `perf/` | Performance improvement |

**Examples**

```
feat/active-hours-scheduling
fix/tray-icon-not-updating-on-wake
chore/bump-tauri-2.11
docs/contributing-guide
refactor/timer-state-machine
```

- Use lowercase, hyphens only (no underscores, no slashes in the description part)
- Keep it short — 3–5 words is ideal
- One concern per branch

---

## Commit messages

Follow **Conventional Commits** — this is what drives automatic version bumping in CI.

```
<type>(<scope>): <short summary>

[optional body]

[optional footer]
```

### Types and their version impact

| Type | Bump | Use for |
|---|---|---|
| `fix` | patch | Bug fixes |
| `feat` | minor | New features |
| `feat!` or `BREAKING CHANGE` in footer | major | Breaking changes |
| `chore` | none | Deps, tooling, CI |
| `docs` | none | Documentation |
| `refactor` | none | Code cleanup |
| `perf` | none | Performance |
| `test` | none | Tests |
| `style` | none | Formatting |

### Scope (optional but encouraged)

Use the module or area affected: `timer`, `tray`, `overlay`, `prefs`, `stats`, `db`, `idle`, `schedule`, `ci`, `deps`.

### Rules

- Summary line: imperative mood, lowercase, no period, ≤72 chars
- Body: explain *why*, not *what* — the diff shows what
- One logical change per commit

### Examples

```
fix(tray): icon not swapping when timer pauses

feat(prefs): add active hours day picker

feat(overlay)!: change opacity range from 0-100 to 50-95

BREAKING CHANGE: existing settings with opacity < 50 will be clamped.

chore(deps): bump tauri to 2.11.0

docs: add contributing guide
```

---

## Pull requests

- Branch → PR → squash merge into `main`
- PR title follows the same conventional commit format (it becomes the squash commit message)
- Keep PRs small and focused — one feature or fix per PR
- Link any related issue in the PR description

---

## Releases

Fully automated. Every merge to `main` triggers CI which:

1. Reads commits since the last tag
2. Bumps the version (`fix` → patch, `feat` → minor, `feat!`/`BREAKING CHANGE` → major)
3. Creates a git tag (`v0.2.0` etc.)
4. Builds macOS universal DMG + Windows installer
5. Opens a draft GitHub release — review artifacts, then publish

You never create tags or releases manually.

---

## Quick reference

```bash
# Start a feature
git checkout main && git pull
git checkout -b feat/my-feature

# Commit
git add src/features/overlay/Overlay.tsx
git commit -m "feat(overlay): add countdown timer to break screen"

# Push and open PR
git push -u origin feat/my-feature
gh pr create
```
