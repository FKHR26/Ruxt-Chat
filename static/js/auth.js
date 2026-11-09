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

document.getElementById('loginBtn').onclick = () => authRequest(`${SERVER_URL}/api/auth/login`);
document.getElementById('signupBtn').onclick = () => authRequest(`${SERVER_URL}/api/auth/signup`);

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
