# user-authentication-system — CLAUDE.md

## What This Project Is

A Rust/axum REST API for user authentication — register and login endpoints. Passwords hashed with bcrypt, JWT returned on login. Any extra fields sent to `/users/register` beyond the required three are stored in MongoDB alongside the standard fields.

Previously this was a Node.js/EJS learning project (files still present: `app.js`, `views/`, `public/`). The Rust API replaced it entirely.

---

## Tech Stack

| Layer | Tech |
|-------|------|
| Runtime | Rust (edition 2024, stable) |
| Framework | axum 0.7 |
| Database | MongoDB via `mongodb` 3.7 crate (`user_auth` DB — `users`, `watch_progress`) |
| Password Hashing | `bcrypt` (cost 12) |
| Auth Tokens | `jsonwebtoken` 9 — HS256, 24h expiry |
| CORS | `tower-http` 0.6 — open (`*`) |
| Config | `dotenv` |

---

## Project Structure

```
user-authentication-system/
├── Cargo.toml
├── .env                    # local only — gitignored
├── .env.example            # safe to commit
├── src/
│   ├── main.rs             # AppState, router, port bind
│   ├── controllers/
│   │   ├── mod.rs
│   │   │   ├── users.rs        # register + login handlers
│   │   └── progress.rs     # watch-progress save + list (JWT-guarded)
│   └── services/
│       ├── mod.rs
│       ├── database_service.rs  # MongoDB Collection<Document> wrapper
│       └── auth_service.rs     # JWT creation + verification
├── .claude/history/        # changeset logs
├── app.js                  # old Node.js (reference only)
├── views/                  # old EJS templates (reference only)
└── public/                 # old CSS (reference only)
```

---

## API Endpoints

### `POST /progress`  *(requires `Authorization: Bearer <jwt>`)*

Upserts the caller's position in one episode, keyed on
(username, show, path). Stored in `user_auth.watch_progress`.

```json
{
  "show": "GOT/Season 03",
  "path": "S03E02.mkv",
  "position": 420.5,
  "duration": 3000
}
```

`show` is the watch key — the bare show name for flat shows, `show/season`
for seasonal ones. `finished` is optional; when omitted it is derived from
`position / duration >= 0.9`.

### `GET /progress`  *(requires bearer token)*

Newest episode per show — one row each, for a "continue watching" list.

### `GET /progress?show=X`  *(requires bearer token)*

Every episode watched in show X, newest first.

All three return 401 on a missing, malformed, or expired token.

Indexes on `watch_progress`: unique (username, show, path), plus
(username, updatedAt desc).

---

### `POST /users/register`

```json
{
  "username": "gaurav",
  "password": "secret",
  "email": "gaurav@example.com",
  "anyExtraField": "stored as-is"
}
```

- Required: `username`, `password`, `email`
- All additional fields in the body are stored in MongoDB alongside the required ones (no schema restriction)
- Password hashed with bcrypt before storing
- Returns `201` + `{ "message": "user created", "id": "<ObjectId>" }`
- Returns `409` if username already exists
- Returns `400` for missing required fields

### `POST /users/login`

```json
{
  "username": "gaurav",
  "password": "secret"
}
```

- Returns `200` + `{ "token": "<jwt>" }` on success
- Returns `401` on wrong credentials

---

## Environment Variables

```env
PORT=4183
MONGODB_URI=mongodb+srv://<user>:<pass>@cluster.mongodb.net/
JWT_SECRET=<long_random_string>
```

---

## Build & Run

```bash
# build
~/.cargo/bin/cargo build

# run (loads .env automatically)
~/.cargo/bin/cargo run

# release binary
~/.cargo/bin/cargo build --release
./target/release/user_auth_api
```

Server binds `0.0.0.0:4183` by default (4179 belongs to the shows-app video server — do not reuse).

---

## Deployment on Pi

Deployed as systemd service `user-auth-api.service` (runs the release binary, loads
`.env` via `EnvironmentFile`, `PORT=4183`). Consumed by `mecca-api-project`
(api-nexus, port 4181) through its `AUTH_API_URL` env var. No tunnel ingress —
internal only.
