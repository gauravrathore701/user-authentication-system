# CLAUDE.md Created

**Date:** 2026-06-22
**Action:** Thorough project analysis + CLAUDE.md written.

## What was done
- Read all source files: app.js, all 5 EJS views, style.css, package.json, README.md, readME.txt
- Analyzed git history (20+ commits on main, develop branch exists)
- Identified key architecture, routes, env requirements, and known issues
- Wrote CLAUDE.md at project root documenting everything

## Key findings
- This is a learning project from earlier in Gaurav's dev journey (Render deploy, curiouslydeveloping.in domain)
- Not deployed on the Pi — no systemd service, no .env present
- Passwords stored in plaintext, no sessions, no real forgot-password flow
- Redundant dual MongoDB connection (MongoClient + Mongoose both used)
- SMTP via Hostinger on old domain noreply@curiouslydeveloping.in
