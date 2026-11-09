const tabButtons = [...document.querySelectorAll('.tab-btn')];
const tabPages = [...document.querySelectorAll('.tab-page')];
const tabBar = document.getElementById('tabBar');
const enterFromLeftClass = 'tab-page--enter-left';
const enterFromRightClass = 'tab-page--enter-right';
let activePageAnimation = null;

tabButtons.forEach((btn, index) => {
  btn.onclick = () => {
    const activeIndex = tabButtons.findIndex(button => button.classList.contains('active'));
    if (activeIndex === index) return;

    if (activePageAnimation) {
      activePageAnimation.page.removeEventListener('animationend', activePageAnimation.onEnd);
      activePageAnimation.page.classList.remove(activePageAnimation.className);
      activePageAnimation = null;
    }

    tabButtons.forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    tabBar.style.setProperty('--active-tab-offset', `${index * 100}%`);

    tabPages.forEach(page => {
      page.style.display = 'none';
      page.classList.remove(enterFromLeftClass, enterFromRightClass);
    });

    const page = document.getElementById('tab-' + btn.dataset.tab);
    page.style.display = 'flex';

    const enterClass = index < activeIndex ? enterFromLeftClass : enterFromRightClass;
    page.classList.add(enterClass);

    if (!window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      const onEnd = event => {
        if (event.target !== page) return;
        page.classList.remove(enterClass);
        page.removeEventListener('animationend', onEnd);
        activePageAnimation = null;
      };
      activePageAnimation = { page, className: enterClass, onEnd };
      page.addEventListener('animationend', onEnd);
    }
  };
});