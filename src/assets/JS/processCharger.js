async function chargerProcessus() {
    const container = document.getElementById('liste-proc');
    
    // 1. On récupère invoke (vérifie que Tauri est chargé)
    if (!window.__TAURI__ || !window.__TAURI__.core) {
        container.innerHTML = "Erreur : Le moteur Tauri n'est pas encore prêt.";
        return;
    }

    const { invoke } = window.__TAURI__.core;
    container.innerHTML = "<i>Recherche des processus...</i>";
    
    try {
        // CORRECTION ICI : On utilise l'underscore '_' comme dans le main.rs
        const procs = await invoke('lister_processus');
        
        if (procs.length === 0) {
            container.innerHTML = "Aucun processus trouvé.";
        } else {
            // On affiche la liste proprement
            container.innerHTML = procs.join('<br>');
        }
    } catch (err) {
        // Si l'erreur est "Command not found", c'est qu'il manque l'autorisation
        container.innerHTML = "<span style='color:red'>Erreur : " + err + "</span>";
        console.error("Détails de l'erreur processus :", err);
    }
}

async function tuer(pid) {
    if (confirm(`Voulez-vous vraiment arrêter le processus ${pid} ?`)) {
        const { invoke } = window.__TAURI__.core;
        const success = await invoke('tuer_processus', { pid: parseInt(pid) });
        if (success) {
            alert("Processus arrêté !");
            chargerProcessus(); // On rafraîchit la liste
        } else {
            alert("Erreur : Impossible d'arrêter ce processus.");
        }
    }
}

function arreterScan() {
    const container = document.getElementById('liste-proc');
    container.innerHTML = "Scan arrêté.";
}

