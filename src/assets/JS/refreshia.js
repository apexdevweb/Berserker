async function refreshAI() {
    const btn = document.querySelector('.clean-btn'); // Ou ton bouton de refresh
    const chat = document.getElementById("chat");
    
    chat.innerHTML += `<div class="msg ia">🔄 Réinitialisation du noyau Ollama...</div>`;
    
    try {
        const res = await invoke('relancer_ollama');
        chat.innerHTML += `<div class="msg ia" style="color:#00ff41">${res}</div>`;
    } catch (err) {
        chat.innerHTML += `<div class="msg ia" style="color:#ff4444">Erreur : ${err}</div>`;
    }
}