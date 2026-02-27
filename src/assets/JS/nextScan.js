async function lancerNextScan() {
    const valInput = document.getElementById('valeur-scan');
    const res = document.getElementById('res-scan');
    const val = parseInt(valInput.value);

    res.innerHTML = "🔍 Filtrage en cours...";

    try {
        const adresses = await invoke('next_scan', { 
            pid: targetPid, 
            nouvelle_valeur: val 
        });

        if (adresses.length === 0) {
            res.innerHTML = "❌ Plus aucun résultat correspondant.";
        } else {
            res.innerHTML = `🎯 Restant : ${adresses.length} adresses<br>` + 
                adresses.map(a => {
                    const hex = `0x${a.toString(16).toUpperCase()}`;
                    return `<div onclick="modifierAdresse(${a}, '${hex}')" style="cursor:pointer; color:#10b981;">${hex} ✏️</div>`;
                }).join('');
        }
    } catch (err) {
        res.innerHTML = "Erreur : " + err;
    }
}
