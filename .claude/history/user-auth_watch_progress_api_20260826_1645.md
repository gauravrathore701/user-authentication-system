# user-authentication-system — Watch Progress API

**Date:** 2026-08-26 16:45 IST

## Why
shows-app tracked "last watched" in `localStorage` only — no memory across
devices. Progress now lives per-user in Mongo, behind the existing JWT.

## What changed

### New file
- `src/controllers/progress.rs` — `save` (POST) + `list` (GET) handlers,
  bearer-token extraction, JWT verification, upsert, latest-per-show dedupe.

### Modified
- `src/services/auth_service.rs` — added `verify_token()` (HS256 decode,
  same secret as `create_token`). Returns `Claims`, `sub` = username.
- `src/services/database_service.rs` — added `watch_progress` collection.
- `src/controllers/mod.rs` — `pub mod progress;`
- `src/main.rs` — route `/progress` (POST save, GET list).
- `Cargo.toml` — added `futures-util = "0.3"` (cursor `try_collect`).

## Endpoints (port 4183)
All require `Authorization: Bearer <jwt>`. 401 on missing/invalid/expired.

- `POST /progress` — body `{show, path, position, duration, finished?}`
  Upserts on (username, show, path). `finished` defaults to
  `position/duration >= 0.9`; an explicit client flag wins.
- `GET /progress` — newest episode per show (one row each).
- `GET /progress?show=X` — every episode watched in show X, newest first.

## Mongo — `user_auth.watch_progress`
```
{ username, show, path, position,
  duration, finished, updatedAt }
```
`show` is the watch key: `"One Piece"` for flat shows,
`"GOT/Season 03"` for seasonal. `path` is the episode filename.

Indexes created manually:
- `uniq_user_show_path` — unique (username, show, path)
- `user_recent` — (username, updatedAt desc)

## Verified
Registered a throwaway user, saved 3 rows, re-saved one to confirm upsert
(not duplicate) and the finished-ratio flip, read back latest-per-show and
single-show, checked 401 on missing and garbage tokens. Test user and its
4 rows deleted afterwards — `watch_progress` left empty.
