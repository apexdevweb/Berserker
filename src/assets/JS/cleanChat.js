function cleanChat() {
    const chat = document.getElementById("chat");
    if (chat) {
        chat.innerHTML = `<div class="msg ia"><i>Terminal purgé. Berserker A.I prêt.</i></div>`;
    }
}