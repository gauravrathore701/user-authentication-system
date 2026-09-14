# README rewritten for the Rust service (2026-09-13 17:15)

- Old README described the retired Node/Express/EJS app (Atlas, Nodemailer).
- New README documents what actually runs: Rust (edition 2024) axum service behind
  the Mecca API, endpoints /users/register, /users/login, /progress (POST, GET, ?show=)
  with statuses, config (MONGODB_URI, JWT_SECRET required; PORT default 4183),
  DB user_auth (users, watch_progress), build/run, systemd unit, layout, and a
  "Legacy" note that app.js/views/public/package.json/readME.txt are unused.
- Links the public docs page https://cursedshrine.com/projects/user-auth-api/.
- Backup: `.claude/backups/README.md.bak-20260913`.
- Not committed by me; the 08:00 git_sync will commit/push (repo is public).
