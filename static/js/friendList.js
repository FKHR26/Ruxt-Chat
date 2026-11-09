async function loadFriends() {
  const res = await fetch(`${SERVER_URL}/api/friends?token=${authToken}`);
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