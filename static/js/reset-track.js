function resetMessage() {
    let messageBody = document.getElementById("message-body");
    Array.from(messageBody.getElementsByClassName("image")).forEach((el) => {
        el.classList.add("is-skeleton");
    });
    Array.from(messageBody.getElementsByClassName("media-content")).forEach((el) => {
        el.classList.add("is-skeleton");
    });
    document.getElementById("reload").style.display = "none";
}

function resetMessageReload() {
    document.getElementById("reload").style.display = "inherit";
}