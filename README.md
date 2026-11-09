# Ruxt-Chat

A messaging app built from scratch in Rust — a simple, modern alternative to building on top of someone else's chat SDK. Friends, direct messages, and a WhatsApp-style tabbed interface, with a web client and a native desktop/mobile client built on the same backend.

## Features

- 🔐 **Accounts** — sign up and log in with a username and password (Argon2-hashed, session tokens)
- 👥 **Friends** — add anyone by username; friendship and a conversation are created automatically
- 💬 **Direct messages** — real-time chat over WebSockets, with full message history saved to Postgres
- 📱 **Responsive UI** — a WhatsApp-style three-tab layout (Chats / Status / Calls) that adapts between a two-pane desktop view and a full-screen mobile view
- 🖥️ **Native client** — the same frontend wrapped with [Tauri](https://tauri.app), buildable as a Windows/macOS/Linux desktop app

## Tech stack

| Layer | Technology |
|---|---|
| Backend | Rust, [Axum](https://github.com/tokio-rs/axum), [Tokio](https://tokio.rs) |
| Database | PostgreSQL via [SQLx](https://github.com/launchbadge/sqlx) |
| Auth | Argon2 password hashing, random session tokens |
| Frontend | Plain HTML / CSS / JavaScript (no framework) |
| Native client | [Tauri](https://tauri.app) |
| CI | GitHub Actions (`.github/workflows/build-client.yml`) |

## Running the server locally

1. Make sure PostgreSQL is installed and running.
2. Create a database and set `DATABASE_URL` in a `.env` file (see `.env.example`):
   ```
   DATABASE_URL=postgres://postgres:postgres@localhost:5432/ruxt_chat
   ```
3. Run it:
   ```
   cargo run
   ```
   Migrations apply automatically on startup. The server listens on port `3000` by default, or on the port given by the `PORT` environment variable (used by hosts like Railway/Render).
4. Open `http://localhost:3000` in a browser.

## Running the native client

The client in `ruxt-chat-client/` is a thin Tauri wrapper around `static/`. It talks to the server over the network, so the server above must be running and reachable first.

1. Set the server's address in `static/js/config.js`:
   ```js
   const SERVER_URL = "https://your-server-address";
   ```
2. From `ruxt-chat-client/`:
   ```
   npm install
   npm run tauri dev     # run in a window
   npm run tauri build   # produce an installer
   ```

GitHub Actions also builds this automatically for Windows, macOS, and Linux on every push to `ruxt-chat-client/` — see the **Actions** tab for downloadable builds.

## Roadmap

- [ ] Permanent hosting for the server (currently a dev Codespace)
- [ ] Status and Calls tabs (currently placeholders)
- [ ] Android/iOS builds via Tauri Mobile
