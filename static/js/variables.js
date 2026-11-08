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
const backBtn = document.getElementById('backBtn');   // NEW
const appRoot = document.getElementById('app');       // NEW

const addFriendModal = document.getElementById('addFriendModal');
const friendUsernameInput = document.getElementById('friendUsername');
const addFriendError = document.getElementById('addFriendError');