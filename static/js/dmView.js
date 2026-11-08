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
  appRoot.classList.add('chat-open'); // NEW: on mobile, slides the chat view over the list
  renderFriends(); // from sidebar.js — re-render so the clicked row highlights

  const res = await fetch(`/api/conversations/${conversationId}/history?token=${authToken}`);
  const history = await res.json();
  history.forEach(renderMessage);

  if (ws) ws.close();
  ws = new WebSocket(`ws://${location.host}/ws/${conversationId}?token=${authToken}`);
  ws.onmessage = (event) => renderMessage(JSON.parse(event.data));
}

// NEW: back button handler — on mobile this reveals the chat list again.
// On desktop both panes are visible anyway, so this just closes the chat.
backBtn.onclick = () => {
  appRoot.classList.remove('chat-open');
  dmView.style.display = 'none';
  emptyState.style.display = 'flex';
  if (ws) { ws.close(); ws = null; }
  activeConversationId = null;
  renderFriends();
};

function sendMessage() {
  if (input.value.trim() && ws) {
    ws.send(input.value.trim());
    input.value = '';
  }
}

input.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') sendMessage();
});

document.getElementById('sendBtn').onclick = sendMessage;