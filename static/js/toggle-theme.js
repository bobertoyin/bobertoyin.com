function toggleTheme() {
    var currentTheme = document.documentElement.getAttribute("data-theme");
    const toggleIcon = document.getElementById("theme-toggle-icon");
    if (currentTheme === null) {
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
            currentTheme = "dark";
        } else {
            currentTheme = "light";
        }
    }
    if (currentTheme === "dark") {
        document.documentElement.setAttribute("data-theme", "light");
        localStorage.setItem("theme", "light");
        toggleIcon.classList.remove("ph-sun");
        toggleIcon.classList.add("ph-moon");
    } else {
        document.documentElement.setAttribute("data-theme", "dark");
        localStorage.setItem("theme", "dark");
        toggleIcon.classList.remove("ph-moon");
        toggleIcon.classList.add("ph-sun");
    }
}