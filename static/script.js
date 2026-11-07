let ws = null;
let servers = [];
let activeServer = null;
let activeChannel = null;
let authToken = null;

const authScreen = document.getElementById('authScreen');
const appEl = document.getElementById('app');
const authUsername = document.getElementById('authUsername');
const authPassword = document.getElementById('authPassword');
const authError = document.getElementById('authError');

const serversEl = document.getElementById('servers');
const channelListEl = document.getElementById('channelList');
const messagesEl = document.getElementById('messages');
const input = document.getElementById('msgInput');

async function authRequest(path) {
  authError.textContent = '';
  const res = await fetch(path, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username: authUsername.value, password: authPassword.value }),
  });
  const data = await res.json();
  if (data.error) {
    authError.textContent = data.error;
    return;
  }
  authToken = data.token;
  sessionStorage.setItem('token', authToken);
  authScreen.style.display = 'none';
  appEl.style.display = '';
  loadServers();
}

document.getElementById('loginBtn').onclick = () => authRequest('/api/auth/login');
document.getElementById('signupBtn').onclick = () => authRequest('/api/auth/signup');

// Note: sessionStorage clears when the tab closes. That's fine for now —
// it just means logging in again each new tab/session.
const savedToken = sessionStorage.getItem('token');
if (savedToken) {
  authToken = savedToken;
  authScreen.style.display = 'none';
  appEl.style.display = '';
  loadServers();
}

async function loadServers() {
  const res = await fetch('/api/servers');
  servers = await res.json();
  renderServers();
  if (servers.length) selectServer(servers[0]);
}

function renderServers() {
  serversEl.innerHTML = '';
  servers.forEach(server => {
    const icon = document.createElement('div');
    icon.className = 'server-icon' + (server === activeServer ? ' active' : '');
    icon.textContent = server.name.slice(0, 2).toUpperCase();
    icon.onclick = () => selectServer(server);
    serversEl.appendChild(icon);
  });
}

function selectServer(server) {
  activeServer = server;
  renderServers();
  channelListEl.innerHTML = '';
  server.channels.forEach(channel => {
    const div = document.createElement('div');
    div.className = 'channel' + (channel === activeChannel ? ' active' : '');
    div.textContent = '# ' + channel.name;
    div.onclick = () => selectChannel(channel);
    channelListEl.appendChild(div);
  });
  if (server.channels.length) selectChannel(server.channels[0]);
}

function renderMessage(msg) {
  const div = document.createElement('div');
  div.className = 'msg';
  div.innerHTML = `<span class="user">${msg.username}</span>${msg.content}`;
  messagesEl.appendChild(div);
  messagesEl.scrollTop = messagesEl.scrollHeight;
}

async function selectChannel(channel) {
  activeChannel = channel;
  messagesEl.innerHTML = '';
  input.placeholder = 'Message #' + channel.name;

  const res = await fetch(`/api/channels/${channel.id}/history`);
  const history = await res.json();
  history.forEach(renderMessage);

  if (ws) ws.close();
  ws = new WebSocket(`ws://${location.host}/ws/${channel.id}?token=${authToken}`);
  ws.onmessage = (event) => {
    renderMessage(JSON.parse(event.data));
  };
}

input.addEventListener('keydown', (e) => {
  if (e.key === 'Enter' && input.value.trim() && ws) {
    ws.send(input.value.trim());
    input.value = '';
  }
});
