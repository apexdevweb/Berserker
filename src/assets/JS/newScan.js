/**
 * Réinitialise le scanner de mémoire pour repartir sur une base propre.
 * Vide la liste côté Rust et nettoie l'interface HTML.
 */
 async function lancerNouveauScan() {
    try {
        // 1. Appel au backend pour vider le vecteur d'adresses
        await window.invoke("nouveau_scan");

        // 2. Nettoyage des variables locales JS
        if (window.dernieresAdresses) {
            window.dernieresAdresses = [];
        }

        // 3. Réinitialisation de l'interface utilisateur
        const resContainer = document.getElementById("res-scan");
        const progressBar = document.getElementById("scan-bar");
        const progressMeter = document.getElementById("progress-meter");
        const valInput = document.getElementById("valeur-scan");

        // On vide les résultats
        if (resContainer) resContainer.innerHTML = "";
        
        // On remet la barre de progression à zéro
        if (progressBar) progressBar.value = 0;
        
        // On remet le texte de progression à l'état initial
        if (progressMeter) {
            progressMeter.innerText = "";
            progressMeter.style.color = "#3b82f6";
        }

        // On vide l'input de recherche pour la clarté
        if (valInput) valInput.value = "";

        console.log("Berserker: Scan réinitialisé, prêt pour une nouvelle recherche.");
        
    } catch (err) {
        console.error("Erreur lors du New Scan:", err);
        alert("Erreur lors de la réinitialisation : " + err);
    }
}

// Exposition pour le bouton HTML (onclick="lancerNouveauScan()")
window.lancerNouveauScan = lancerNouveauScan;

