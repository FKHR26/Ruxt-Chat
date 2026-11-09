// IMPORTANT: set this to your server's actual address (no trailing slash).
const SERVER_URL = "https://refactored-space-sniffle-7vw9rr6qxx6c66x-3000.app.github.dev".replace(/\/+$/, "");

// Automatically picks ws:// or wss:// based on whether SERVER_URL itself is https.
const WS_BASE = SERVER_URL.replace(/^http/, "ws");
