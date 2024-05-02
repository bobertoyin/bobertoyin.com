function initTheme() {
    const theme = localStorage.getItem("theme");
    if (theme !== null) {
        document.documentElement.setAttribute("data-theme", theme);
    }
};

function initThemeIcon() {
    var theme = localStorage.getItem("theme");
    if (theme === null) {
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
            theme = "dark";
        } else {
            theme = "light";
        }
    }
    var toggleIcon = document.getElementById("theme-toggle-icon");
    if (theme === "dark") {
        toggleIcon.classList.remove("ph-moon");
        toggleIcon.classList.add("ph-sun");
    } else {
        toggleIcon.classList.remove("ph-sun");
        toggleIcon.classList.add("ph-moon");
    }
}

initTheme();
window.onload = initThemeIcon;