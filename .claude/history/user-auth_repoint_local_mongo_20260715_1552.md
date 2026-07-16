# user-auth-api — repointed from Atlas to local Pi Mongo

**Date:** 2026-07-15 15:52
**Request:** Point user-auth-api at the local Pi Mongo (with auth) instead of MongoDB Atlas.

## Before
- `.env` `MONGODB_URI` → **MongoDB Atlas** `mongodb+srv://admin:***@pidb.jjyttpq.mongodb.net`
- DB `user_auth`, collection `users`.

## What was done
1. **Attempted to migrate Atlas data → local — Atlas was UNREACHABLE** from the Pi (SSL/`ssl3_read_bytes` alert → Network Access List; free-tier clusters also auto-pause). Could not carry over existing accounts. Gaurav had already said data isn't a concern, so proceeded with a fresh local `user_auth` DB.
2. Created least-privilege Mongo user `auth_app` — `readWrite@user_auth` only (root-authenticated). Creds saved to `mongo-pi/.env` (chmod 600, gitignored).
3. Repointed `user-authentication-system/.env` `MONGODB_URI` → `mongodb://auth_app:***@localhost:27017/?authSource=admin`. Old Atlas URI kept commented in `.env` for future migration. `.env` chmod 600, already gitignored.
4. `systemctl restart user-auth-api.service`.

## Verified end-to-end
- Service **active**, "running on port 4183", no errors.
- `POST /users/register` → **HTTP 201**, doc landed in **local** `user_auth.users` (confirmed via `auth_app`).
- `POST /users/login` → **200 + JWT**.
- Temp `__healthcheck__` user deleted afterward (users count back to 0).

## Notes / follow-ups
- **Old Atlas accounts NOT migrated** (cluster unreachable). If needed later: whitelist the Pi's IP in Atlas (or unpause the cluster), then `mongodump` Atlas `user_auth` → `mongorestore` into local. Old URI is in the `.env` comment.
- Local `user_auth` starts empty — new registrations go to the Pi now.
- `mecca-api-project` (api-nexus, 4181) consumes this API via `AUTH_API_URL` — unaffected (talks HTTP to 4183, not Mongo directly).
- Atlas credential (`admin:keJuzYi0…`) is now unused by this service but still sits in the `.env` comment + Atlas itself — rotate/decommission the Atlas cluster if fully done with it.
