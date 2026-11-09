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
  const res = await fetch(`${SERVER_URL}/api/friends`, {
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