document.addEventListener('DOMContentLoaded', () => {
    const burgerElements = document.querySelectorAll(".navbar-burger");
    burgerElements.forEach(burger => {
        burger.addEventListener("click", () => {
            const target = document.getElementById(burger.dataset.target);
            burger.classList.toggle("is-active");
            target.classList.toggle("is-active");
        });
    });
});