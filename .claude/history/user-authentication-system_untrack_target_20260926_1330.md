# Untracked `target/` from git

**Date:** 2026-09-26 13:30 IST

`target/release/user_auth_api` (12.3 MB) was tracked, along with 1702
other build-output files — fingerprints and timestamps that churn on
every single build.

```
git rm -r --cached target/
echo /target/ >> .gitignore
```

1703 deletions staged, **not committed** — the 08:00 cron
`~/routines/git_sync.sh` commits and pushes everything staged and
untracked on `develop`, so tomorrow's run carries this along with
the `isUniversalUser` change and the history docs.

Nothing left the disk: the binary is still there and `user-auth-api` is
still `active`. Only the index changed.

Note the old blobs stay in git history — untracking stops future growth,
it does not shrink the existing repo. Rewriting history would, and that
has not been done.

## Staged for tomorrow's sync

```
M  .gitignore
M  src/controllers/users.rs
D  target/… (1703)
?? .claude/history/*_account_merge_*
?? .claude/history/*_universal_user_flag_*
?? .claude/history/*_untrack_target_*
```
