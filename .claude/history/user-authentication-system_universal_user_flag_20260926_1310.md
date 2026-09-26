# `isUniversalUser` — master account flag

**Date:** 2026-09-26 13:10 IST

## What it does

A user document carrying `isUniversalUser: true` logs in to **any**
allowed client without being listed in its `clients` array. The password
is the only check for that user.

`src/controllers/users.rs`:

```rust
fn is_universal(user: &bson::Document) -> bool {
    user.get_bool("isUniversalUser").unwrap_or(false)
}
```

and in `login`, the membership test became:

```rust
if !is_universal(&user)
    && !clients_of(&user).contains(&client) {
    return unauthorized();
}
```

Absent field reads as `false`, so every existing user keeps the old
behaviour with no migration.

## What it deliberately does NOT bypass

The `X-Client-Id` middleware is untouched. A request still has to name a
**known** client — missing header is 400, unknown client is 403, for a
universal user exactly as for anyone else. The flag only skips the
per-user membership test, not the tenant allowlist.

The JWT still carries the `client` claim of whichever client the login
came through, so `/users/verify` and `/progress` stay strictly
per-tenant: a `files_project` token is still 403 against
`shows_project`. Universality is about *where you may log in*, not about
one token opening everything. Say the word if you want the claim check
relaxed too — it is a separate change.

## Verified

Flag proved by temporarily stripping `shows_project` out of `gaurav`'s
`clients`, logging in anyway, then restoring the array:

```
shows login, not in clients  200  <- flag
files login                  200
wrong password               401
unknown client id            403
missing client header        400
```

Gate still holds for normal users — temp account `flagprobe`
(files_project only, deleted afterwards):

```
flagprobe -> files_project   200
flagprobe -> shows_project   401
```

End to end, nothing regressed:

```
mecca /api/auth/login        200
files.cursedshrine.com/      200
files token vs shows verify  403
/progress rows               12 shows
```

Final state of the collection — one document:

```
gaurav
  clients ["files_project",
           "shows_project"]
  isUniversalUser true
```

## Notes

- Built with `cargo build --release` (cargo is not on the default PATH;
  it lives in `~/.cargo/bin`), then `sudo systemctl restart
  user-auth-api`. **Never** `pkill -f target/release/user_auth_api` —
  that is the live unit's binary path.
- `clients` is now redundant for `gaurav` but was left in place; it is
  what a future non-universal account will use, and `/progress` does not
  read it.
