# user-authentication-system — history cleanup + push fix (2026-06-29)

## What happened
- Push was failing because GitHub rejected two committed Rust build artifacts in `target/debug/` over the 100MB limit:
  - `target/debug/deps/libmongodb-22e5c1e974c74105.rlib` (139 MB)
  - `target/debug/deps/user_auth_api-918142be26c76d41` (124 MB)
- 1923 files under `target/` were tracked across 26 unpushed commits on `develop`, despite `target/` later being added to `.gitignore` (gitignore alone doesn't remove already-committed files from history).
- Ran `git filter-repo --path target --invert-paths --force` to strip `target/` from all history.
  - `.git` size: 292MB → 5.1MB
  - This rewrote commit hashes for all local commits (expected/required side effect, flagged to user beforehand).
- `filter-repo` auto-removed the `origin` remote (its default safety behavior); re-added it.
- Remote `develop` only had our own pre-rewrite commits under old hashes (no foreign work), so force-pushed safely with `git push --force-with-lease --set-upstream origin develop`.

## Follow-up
- `target/` is in `.gitignore`; no recurrence expected as long as build artifacts aren't manually `git add`ed.
