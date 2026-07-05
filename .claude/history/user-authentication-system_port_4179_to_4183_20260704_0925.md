# Port collision fix: default port 4179 → 4183

**Date:** 2026-07-04 09:25

## Why
4179 is the shows-app video-server's port. The Rust auth API's *default* was
also 4179 — the live service was already overridden to 4183 via `.env`, but
every fallback/doc still said 4179, so anything run without the env override
(or api-nexus with `AUTH_API_URL` unset) would hit the video server.

## Changes
- `src/main.rs` — default port fallback `4179` → `4183`; release binary rebuilt.
- `.env.example`, `CLAUDE.md` — PORT/bind docs updated to 4183; deployment
  section updated (it IS deployed: systemd `user-auth-api.service`).
- `mecca-api-project` — `AUTH_API_URL` default `localhost:4179` → `localhost:4183`
  in `application.properties`, `.env.example`, `CLAUDE.md`; jar rebuilt.
- Both services restarted.

## Verification
- `user_auth_api` listening on 4183, video-server still on 4179, api-nexus on 4181.
- `POST :4181/api/auth/login` (bad creds) → proper 401 proxied from
  `http://localhost:4183/users/login`. Direct `:4183/users/login` → 401 too.
- Zero `4179` references remain in either project's configs/docs.
