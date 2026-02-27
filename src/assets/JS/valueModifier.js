async function modifierAdresse(addrInt, addrHex) {
    const nouvelleVal = prompt(`Quelle nouvelle valeur pour ${addrHex} ?`, "999");
    if (nouvelleVal === null) return;

    try {
        const message = await invoke('ecrire_valeur_memoire', { 
            pid: targetPid, 
            adresse: addrInt, 
            nouvelleValeur: parseInt(nouvelleVal) 
        });
        alert(message);
    } catch (err) {
        alert("Erreur : " + err);
    }
}