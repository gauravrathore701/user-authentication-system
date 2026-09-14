# User Auth API

A small Rust service that owns user accounts and watch progress for the
[Cursed Shrine](https://cursedshrine.com) projects on a Raspberry Pi:
bcrypt-hashed passwords, 24-hour JWTs, and a MongoDB store.

Public traffic never hits it directly — it sits behind the **Mecca API** gateway
(`https://api.cursedshrine.com/api`), which forwards `/auth/*` and `/progress` here.

Docs page: https://cursedshrine.com/projects/user-auth-api/

---

## Tech stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (edition 2024) |
| Web framework | axum 0.7 + Tokio |
| Database | MongoDB (official driver 3.x) |
| Passwords | bcrypt |
| Tokens | jsonwebtoken (HS256, 24 h expiry) |
| CORS | tower-http, any origin |

## API

All bodies are JSON. Errors are `{"error": "<reason>"}`.

### `POST /users/register`

```json
{ "username": "gaurav", "email": "you@example.com", "password": "••••••••" }
```

| Status | Body |
|--------|------|
| `201` | `{"message": "user created", "id": "<ObjectId>"}` |
| `400` | `username required` / `email required` / `password required` |
| `409` | `username already exists` |

### `POST /users/login`

```json
{ "username": "gaurav", "password": "••••••••" }
```

| Status | Body |
|--------|------|
| `200` | `{"token": "<JWT>"}` — `sub` = username, expires in 24 h |
| `400` | `username required` / `password required` |
| `401` | `invalid credentials` |

### Watch progress

Cross-device resume for the Shows app. Every route needs
`Authorization: Bearer <token>`; the user always comes from the token, never the body.

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/progress` | Upsert `{show, path, position, duration, finished?}` |
| `GET` | `/progress` | Latest watched episode per show |
| `GET` | `/progress?show=<show>` | All episodes watched in one show, newest first |

`finished` wins if the client sends it; otherwise an episode counts as finished once
`position / duration >= 0.9`.

| Status | Body |
|--------|------|
| `400` | `show and path required` |
| `401` | `missing bearer token` / `invalid or expired token` |

## Configuration

Copy `.env.example` to `.env` (gitignored) and fill it in:

| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `MONGODB_URI` | yes | — | MongoDB connection string (panics if missing) |
| `JWT_SECRET` | yes | — | HMAC secret for signing tokens (panics if missing) |
| `PORT` | no | `4183` | Listen port (binds `0.0.0.0`) |

Data lives in database `user_auth`, collections `users` and `watch_progress`.

## Build and run

```bash
cargo build --release
./target/release/user_auth_api      # reads .env from the working directory
```

On the Pi it runs as `user-auth-api.service`:

```bash
sudo systemctl status user-auth-api
sudo systemctl restart user-auth-api
```

## Project layout

```
src/
├── main.rs                   router, CORS, bind
├── controllers/
│   ├── users.rs              register, login
│   └── progress.rs           watch-progress upsert + queries
└── services/
    ├── auth_service.rs       bcrypt + JWT
    └── database_service.rs   MongoDB collections
```

## Legacy

This repo started as a Node.js/Express login app (EJS views, Nodemailer).
Those files — `app.js`, `views/`, `public/`, `package.json`, `readME.txt` — are
still in the tree for reference but are **not** built or deployed; the running
service is the Rust code in `src/`.
