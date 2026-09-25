# Client (tenant) gate on every auth endpoint

Date: 2026-09-24 19:00 IST

## Ask
Every request names its project in a header. A user under
`shows_project` must not be able to log in to
`files_project`. Middleware in the auth service, client
list hardcoded for now, id sent as a header not in the body.

## Correction to the earlier premise
No client concept existed anywhere before this change —
not in `users.rs`, not in mecca, and the single user doc
had no such field. This built it rather than wired it up.

## New — src/middlewares/client.rs
- `ALLOWED_CLIENTS = ["shows_project", "files_project"]`
- `require_client` axum middleware on **all** routes
  - missing/empty `X-Client-Id` -> 400
  - unknown client              -> 403
  - valid -> `ClientId` in request extensions
- controllers read it via `Extension<ClientId>`

## Data
`users.clients: [String]`. Backfilled the one existing user
to `["shows_project"]` (additive `$addToSet`, nothing
overwritten).

## Endpoints
- `POST /users/register` — writes `clients: [client]`.
  An existing username registering under a **new** client
  gets that client added instead of 409 — but only when the
  password verifies, so it can't be used to attach to
  someone else's account. Wrong password still returns the
  same 409 as before, leaking nothing.
- `POST /users/login` — user must hold the client or 401.
  Deliberately the *same* 401 as a bad password: a distinct
  error would reveal which users belong to which app.
  JWT now carries a `client` claim. Also sets the token as
  `HttpOnly; Secure; SameSite=Lax` cookie `auth_token`, so
  a browser file listing can authenticate without a header.
- `GET /users/verify` — NEW. For Caddy `forward_auth`.
  200 when the token is live **and** its claim matches the
  request's client; 403 on claim mismatch; 401 otherwise.
  Reads `Authorization: Bearer` or the cookie.
- `POST /users/logout` — NEW. Clears the cookie. The JWT
  stays valid till expiry; no revocation list yet.
- `/progress` — `caller()` now also compares the claim to
  the request client, so a files token cannot touch a shows
  watch history.

## Verified live (scratch instance, port 4199)
```
no header                 400
unknown client            403
valid client, bad pw      401
right pw, wrong client    401
register under files      201
login files               200 + cookie
login shows (not held)    401
verify files tok/files    200
verify files tok/shows    403
verify via cookie only    200
verify no token           401
/progress cross-client    403
grant via register+pw     200, login then 200
grant with wrong pw       409, no grant
```
Test user `rextest` created for this and deleted after;
DB is back to the one real user.

## mecca-api — required, done
The gate broke shows-app login the moment it went live.
Fixed in `WebClientConfig`: the `authWebClient` bean now
sends a default `X-Client-Id`, from
`downstream.auth.client-id` (`AUTH_CLIENT_ID`, default
`shows_project`). One place, so no adapter can forget it,
and mecca stays a pure connector — no logic added.
Rebuilt and restarted; `POST /api/auth/login` with a bad
password now returns 401, not 400, proving the header
arrives.

## Incident during this change
`pkill -f "target/release/user_auth_api"` was meant for the
scratch instance but matched `user-auth-api.service`, which
runs the same binary path. The live service went down and
came straight back on the new binary (~10s, NRestarts=0),
which also meant the client gate went live before mecca was
ready. Use `--pidfile` or a distinct binary path next time.

## Not done yet
Caddy login page, `forward_auth` on :4185, tunnel ingress.
:4185 is still LAN-only with no auth.

## Backups
`src/main.rs`, `src/controllers/users.rs`,
`src/controllers/progress.rs`,
`src/services/auth_service.rs` -> `.bak-20260924`
mecca: `WebClientConfig.java`, `application.properties`
-> `.bak-20260924`
