# User Auth API — First-time Setup on Pi

**Date:** 2026-07-01 21:45

## What was done

- Created `.env` with MongoDB URI (piDb cluster) and JWT secret
- Built Rust binary: `~/.cargo/bin/cargo build --release` → `target/release/user_auth_api`
- Ran on **port 4183** (4179 was already taken by shows-app video-server.js)
- Created `/etc/systemd/system/user-auth-api.service` — enabled and active

## mecca-api change

Added `AUTH_API_URL=http://localhost:4183` to `/etc/systemd/system/mecca-api-project.service`
(default was localhost:4179 which conflicted with shows-app)

## shows-app proxy change

`app/api/auth/login/route.js` — changed upstream from `https://api.cursedshrine.com/auth/login`
to `http://localhost:4181/auth/login` (Pi can't resolve its own public domain from inside)
