// IMPORTANT: set this to your server's actual address (no trailing slash).
// - While testing: your Codespace's public forwarded URL
// - Once hosted: your Railway/Render URL (e.g. https://ruxt-chat.up.railway.app)
//
// This is needed because, inside the Tauri desktop window, the page no
// longer loads *from* your server — it loads from a local Tauri address —
// so relative paths like fetch('/api/...') or ws://location.host no longer
// point anywhere useful. Everything must use this full address instead.
const SERVER_URL = "https://refactored-space-sniffle-7vw9rr6qxx6c66x-3000.app.github.dev/";

// Automatically picks ws:// or wss:// based on whether SERVER_URL itself is https.
const WS_BASE = SERVER_URL.replace(/^http/, "ws");