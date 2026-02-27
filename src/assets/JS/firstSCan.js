// --- FONCTION : FIRST SCAN (Version Tauri v2) ---
async function lancerPremierScan() {
    const valInput = document.getElementById('valeur-scan');
    const res = document.getElementById('res-scan');
    
    // Vérification de la cible (définie dans processCharger.js)
    if (!targetPid) {
        alert("⚠️ Sélectionnez une cible dans 'Select Target' d'abord !");
        return;
    }

    const val = parseInt(valInput.value);
    if (isNaN(val)) {
        alert("⚠️ Entrez un nombre valide à rechercher.");
        return;
    }

    res.innerHTML = "🔍 Scanning RAM (Admin requis)...";

    try {
        // CORRECTION CRUCIALE : Tauri v2 attend 'valeurRecherchee' (CamelCase)
        // car il transforme automatiquement 'valeur_recherchee' du Rust.
        const adresses = await invoke('premier_scan', { 
            pid: targetPid, 
            valeurRecherchee: val 
        });

        if (!adresses || adresses.length === 0) {
            res.innerHTML = `❌ Aucun résultat pour ${val} (Vérifiez les droits Admin)`;
        } else {
            // Affichage des adresses trouvées au format HEX (0x...)
            res.innerHTML = `✅ Trouvé : ${adresses.length} adresses<br>` + 
            adresses.map(a => {
                const hex = `0x${a.toString(16).toUpperCase()}`;
                return `<div class="addr-item" onclick="modifierAdresse(${a}, '${hex}')" style="cursor:pointer; color:#4ade80;">${hex} ✏️</div>`;
            }).join('');
        }
    } catch (err) {
        // Si l'erreur persiste, l'erreur s'affichera ici
        res.innerHTML = `<span style="color:red">⚠️ Erreur : ${err}</span>`;
        console.error("Détails de l'erreur :", err);
    }
}

