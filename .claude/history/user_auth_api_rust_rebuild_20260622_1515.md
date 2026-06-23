# user-authentication-system — Rebuilt as Rust API
**Date:** 2026-06-22 15:15

## What Was Done

Replaced the old Node.js/Express/EJS auth app with a Rust/axum REST API. Old Node.js files kept for reference but are no longer used.

## New Stack

| Layer | Tech |
|-------|------|
| Runtime | Rust (stable, edition 2024) |
| Framework | axum 0.7 |
| Database | MongoDB (via mongodb 3.7) |
| Auth | bcrypt (password hashing) + jsonwebtoken 9 (JWT, HS256, 24h expiry) |
| CORS | tower-http 0.6 (allow all origins/methods/headers) |
| Port | 4179 (configurable via PORT env var) |

## API Endpoints

### POST /users/register
Body: `{ "username": "...", "password": "...", "email": "...", ...anyExtraFields }`
- Required: username, password, email
- Any additional JSON fields are stored as-is in MongoDB alongside the required fields
- Passwords hashed with bcrypt (cost 12)
- Returns 201 with `{ "message": "user created", "id": "<mongo_id>" }`
- Returns 409 if username already exists

### POST /users/login
Body: `{ "username": "...", "password": "..." }`
- Returns 200 with `{ "token": "<jwt>" }` on success
- Returns 401 on bad credentials

## File Structure Created

```
src/
├── main.rs                         # AppState, router, bind 0.0.0.0:4179
├── controllers/
│   ├── mod.rs
│   └── users.rs                    # register + login handlers
└── services/
    ├── mod.rs
    ├── database_service.rs          # MongoDB Collection<Document> wrapper
    └── auth_service.rs              # JWT token creation
Cargo.toml
.env.example
```

## Environment Variables Required

```
PORT=4179
MONGODB_URI=mongodb+srv://<user>:<pass>@cluster.mongodb.net/
JWT_SECRET=<long_random_secret>
```

Database: `user_auth`, Collection: `users`

## Build

Compiled successfully with `cargo build` (release target available with `cargo build --release`).
