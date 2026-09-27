# Purged `target/` from all git history

**Date:** 2026-09-26 13:45 IST

```
.git  204M  ->  5.1M
```

## Backup taken first

```
/home/gaurav/backups/
  user-auth-repo-20260926/
    pre-rewrite.bundle   166M
    refs-before.txt
```

The bundle holds **every ref** as it was before the rewrite. Restore with
`git clone pre-rewrite.bundle`. `refs-before.txt` records the old tips:
develop and origin/develop at `cacc33a`, main at `15d6f3e`.

## Two commits made first

A rewrite needs a clean tree, so the work that was deliberately left
staged for tomorrow's cron had to be committed now:

```
feat(auth): add isUniversalUser
             master-account flag
chore: stop tracking build output
       in target/
```

## The rewrite

```
git filter-repo --path target/ \
    --invert-paths --force
```

`--force` alone was not enough: this repo had been filtered once before
(`.git/filter-repo/already_ran`, dated 2026-06-29), which triggers an
interactive "treat as continuation?" prompt. In a non-interactive shell
that surfaces as `EOFError: EOF when reading a line` and **nothing is
changed**. Answer it by piping `y`.

filter-repo also **deletes the `origin` remote** by design, to stop an
accidental push. Re-added afterwards.

## Verified

```
target objects in history   0
biggest remaining blob      4.1 MB
                (node_modules/carlo)
source intact (isUniversalUser x2)
binary still on disk
user-auth-api               active
35 commits parsed, 3 refs
```

## Not finished — needs a force push

Every SHA changed, so local and remote have diverged:

```
remote-only commits  6
local-only commits   8
origin/develop       cacc33a (old)
```

The 08:00 `git_sync.sh` cron does a plain push, which will now **fail as
non-fast-forward**. Until someone runs:

```
git push --force-with-lease \
    origin develop
git push --force-with-lease \
    origin main
```

…GitHub still holds the 204 MB history and the daily sync is broken.
Awaiting Gaurav's go on that.

## Also still in history

2497 objects under `node_modules/` — a leftover from when this repo held
a Node project. The largest single blob in the repo is now a 4.1 MB
Chromium shader cache from `node_modules/carlo`. Stripping those would
need a second `--path node_modules/ --invert-paths` pass. Not done; not
asked for.

---

## Force push done — 13:50 IST

```
git push --force-with-lease origin develop
 + cacc33a...178b0f6 develop
   -> develop (forced update)
```

Remote and local now agree exactly:

```
remote-only commits   0
local-only commits    0
origin/develop        178b0f6
target/ files on remote  0
```

**`main` deliberately not pushed.** See the branch note below.

One wrinkle: the `git fetch` run *before* the push pulled the old 204 MB
objects back down, so `.git` bounced from 5.1 MB to 170 MB. After the
force push those objects are unreachable, so:

```
git reflog expire --expire=now --all
git gc --prune=now
```

`.git` back to **5.1 MB**, `git fsck` clean.

## Why there are two branches

Not something introduced today — `main` is the repo's original 2023
branch and `develop` is what everything since has been built on.

```
local  main    15d6f3e
               "Revert changes"
               2023-10-06
origin/main    d3e3aec
develop        178b0f6  (12 ahead
               of local main)
```

Three separate facts:

1. `develop` is the working branch. `~/routines/git_sync_projects.conf`
   line 34 lists this project with `develop` as its only push target, so
   the daily cron has never touched `main`.
2. Local `main` and `origin/main` are **different commits** and have been
   for a long time. Local main is a stale 2023 tip.
3. `main` was untouched by the rewrite. It branched before `target/` was
   ever committed, so none of its commits contained those blobs and none
   of their SHAs changed.

That is why only `develop` was force-pushed. Pushing local `main` would
overwrite `origin/main` with a three-year-old state, which is a different
operation and was not asked for.

## Backup dropped — 13:55 IST

On Gaurav's say-so, `/home/gaurav/backups/user-auth-repo-20260926/`
was removed (`pre-rewrite.bundle` 166 MB + `refs-before.txt`). The old
204 MB history no longer exists anywhere — not locally, not on GitHub,
not in a bundle. `/home/gaurav/backups/user_auth-20260926/` (the Mongo
dump) is untouched.
