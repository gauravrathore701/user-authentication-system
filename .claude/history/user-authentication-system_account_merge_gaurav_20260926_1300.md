# `user` → `gaurav`: rename, new password, one merged account

**Date:** 2026-09-26 13:00 IST

## Why it became a merge

Gaurav asked to rename `user` to `gaurav` and give it a new password.
`gaurav` already existed from earlier today (`files_project`), and
usernames are the lookup key in `/users/login`, so two docs called
`gaurav` is not a state the service can serve. Resolved by folding
`user` into the existing `gaurav` doc rather than renaming in place.

Result: **one account, both tenants.**

```
gaurav  killingwiz@gmail.com
        files_project
        shows_project
```

## What ran

Backup first, **outside the repo** (`.claude/` is git-ignored here but
bcrypt hashes still don't belong in a project tree):

```
/home/gaurav/backups/user_auth-20260926/
  users.json           4 docs
  watch_progress.json  45 docs
  dir 700, files 600
```

Then, in one mongosh script:

1. `users.updateOne({username:"gaurav"})` — new bcrypt hash,
   `clients: ["files_project","shows_project"]`. matched 1, modified 1.
2. `watch_progress.updateMany({username:"user"} → "gaurav")` —
   matched 45, modified 45. Resume history kept.
3. `users.deleteMany({username: {$in: ["user","verify_probe",
   "verify_probe_shows"]}})` — deleted 3. The two probes were this
   task's own test accounts.

Hash was generated with Python `bcrypt.gensalt(rounds=12)` →
`$2b$12$…`, the same format and cost the Rust side writes
(`bcrypt::DEFAULT_COST` is 12), which the login test below confirms.

**The password is not recorded here or anywhere on disk** — Discord
only. The `files_project` password issued earlier today is dead; this
one replaces it for both projects.

## Verified

```
login files_project   200
login shows_project   200
wrong password        401
/progress (shows tok) 12 shows returned
files.cursedshrine…/  200 browse
mecca /api/auth/login 200
```

`users` now holds exactly one document.

## Worth knowing

- There is still no change-password or rename endpoint on 4183. Both
  operations mean touching Mongo directly, as here.
- One password now opens both shows-app and the file browser. Separate
  credentials would need two usernames, which is a product decision,
  not a limitation.
- `mongosh` lives in the `mongodb` container, not on the host:
  `docker exec -i mongodb mongosh --quiet -u auth_app -p <pw> \
   --authenticationDatabase admin < script.js`
