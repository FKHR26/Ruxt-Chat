let ws = null;
let authToken = null;
let myUsername = null;
let friends = [];
let activeConversationId = null;
let activeFriendUsername = null;

const authScreen = document.getElementById('authScreen');
const appEl = document.getElementById('app');
const authUsername = document.getElementById('authUsername');
const authPassword = document.getElementById('authPassword');
const authError = document.getElementById('authError');

const friendListEl = document.getElementById('friendList');
const emptyState = document.getElementById('emptyState');
const dmView = document.getElementById('dmView');
const dmTitle = document.getElementById('dmTitle');
const messagesEl = document.getElementById('messages');
const input = document.getElementById('msgInput');

const addFriendModal = document.getElementById('addFriendModal');
const friendUsernameInput = document.getElementById('friendUsername');
const addFriendError = document.getElementById('addFriendError');

// ---------- Auth ----------
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
  myUsername = data.username;
  sessionStorage.setItem('token', authToken);
  sessionStorage.setItem('username', myUsername);
  enterApp();
}

document.getElementById('loginBtn').onclick = () => authRequest('/api/auth/login');
document.getElementById('signupBtn').onclick = () => authRequest('/api/auth/signup');

const savedToken = sessionStorage.getItem('token');
if (savedToken) {
  authToken = savedToken;
  myUsername = sessionStorage.getItem('username');
  enterApp();
}

function enterApp() {
  authScreen.style.display = 'none';
  appEl.style.display = '';
  loadFriends();
}

// ---------- Tabs ----------
document.querySelectorAll('.tab-btn').forEach(btn => {
  btn.onclick = () => {
    document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    document.querySelectorAll('.tab-page').forEach(p => p.style.display = 'none');
    document.getElementById('tab-' + btn.dataset.tab).style.display = 'flex';
  };
});

// ---------- Friends list ----------
async function loadFriends() {
  const res = await fetch(`/api/friends?token=${authToken}`);
  friends = await res.json();
  renderFriends();
}

function renderFriends() {
  friendListEl.innerHTML = '';
  if (!friends.length) {
    friendListEl.innerHTML = '<p class="placeholder-text">No chats yet. Tap + to add a friend.</p>';
    return;
  }
  friends.forEach(friend => {
    const row = document.createElement('div');
    row.className = 'friend-row' + (friend.conversation_id === activeConversationId ? ' active' : '');
    row.innerHTML = `
      <div class="friend-avatar">${friend.friend_username.slice(0, 2).toUpperCase()}</div>
      <div class="friend-name">${friend.friend_username}</div>
    `;
    row.onclick = () => openDm(friend.conversation_id, friend.friend_username);
    friendListEl.appendChild(row);
  });
}

// ---------- Add friend ----------
document.getElementById('addFriendBtn').onclick = () => {
  friendUsernameInput.value = '';
  addFriendError.textContent = '';
  addFriendModal.style.display = 'flex';
};
document.getElementById('cancelAddFriend').onclick = () => {
  addFriendModal.style.display = 'none';
};
document.getElementById('confirmAddFriend').onclick = async () => {
  const username = friendUsernameInput.value.trim();
  if (!username) return;
  const res = await fetch('/api/friends', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ token: authToken, username }),
  });
  const data = await res.json();
  if (data.error) {
    addFriendError.textContent = data.error;
    return;
  }
  addFriendModal.style.display = 'none';
  await loadFriends();
};

// ---------- DM view ----------
function renderMessage(msg) {
  const div = document.createElement('div');
  const isMe = msg.sender_username === myUsername;
  div.className = 'bubble ' + (isMe ? 'me' : 'them');
  div.textContent = msg.content;
  messagesEl.appendChild(div);
  messagesEl.scrollTop = messagesEl.scrollHeight;
}

async function openDm(conversationId, friendUsername) {
  activeConversationId = conversationId;
  activeFriendUsername = friendUsername;
  dmTitle.textContent = friendUsername;
  messagesEl.innerHTML = '';
  emptyState.style.display = 'none';
  dmView.style.display = 'flex';
  renderFriends(); // re-render so the clicked row gets the "active" highlight

  const res = await fetch(`/api/conversations/${conversationId}/history?token=${authToken}`);
  const history = await res.json();
  history.forEach(renderMessage);

  if (ws) ws.close();
  ws = new WebSocket(`ws://${location.host}/ws/${conversationId}?token=${authToken}`);
  ws.onmessage = (event) => renderMessage(JSON.parse(event.data));
}

input.addEventListener('keydown', (e) => {
  if (e.key === 'Enter' && input.value.trim() && ws) {
    ws.send(input.value.trim());
    input.value = '';
  }
});