# User Authentication System

A full-stack Node.js web application implementing user registration, login, and email verification — built with Express, EJS templating, MongoDB, and Nodemailer.

> Cloned from GitHub for reference/local development. Not currently deployed on a public URL.

---

## Features

- User registration with email + password
- Login / logout flow
- MongoDB-backed user persistence (Mongoose)
- Email notifications via Nodemailer
- Server-side rendered views using EJS templates
- Color palette: `#F5EBEB` (background) / `#E4D0D0` / `#D5B4B4` / `#867070`

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Runtime | Node.js |
| Framework | Express 4 |
| Templating | EJS |
| Database | MongoDB (Mongoose) |
| Email | Nodemailer |
| Config | dotenv |

## Project Structure

```
user-authentication-system/
├── app.js              # Main Express server + routes
├── views/              # EJS templates (register, login, dashboard)
├── public/             # Static assets (CSS, images)
└── package.json
```

## Setup

1. Create a `.env` file:
```env
PORT=3000
CONNECTIONSTRING=mongodb+srv://<user>:<pass>@cluster.mongodb.net/UserAuthenticationSystem
EMAIL_USER=your@email.com
EMAIL_PASS=your_app_password
```

2. Install dependencies and run:
```bash
npm install
node app.js
```

App runs on `http://localhost:3000` (or `PORT` from `.env`).

## Database

Uses MongoDB Atlas. Collection: `UserAuthenticationSystem.users`

Schema:
```js
{ username: String, email: String, password: String }
```
