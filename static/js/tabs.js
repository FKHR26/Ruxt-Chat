document.querySelectorAll('.tab-btn').forEach(btn => {
  btn.onclick = () => {
    document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    document.querySelectorAll('.tab-page').forEach(p => p.style.display = 'none');
    document.getElementById('tab-' + btn.dataset.tab).style.display = 'flex';
  };
});