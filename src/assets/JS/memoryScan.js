async function lireMemoire() {
    const pidInput = document.getElementById('scan-pid');
    const addrInput = document.getElementById('scan-addr');
    const resultDiv = document.getElementById('res-mem');

    const pid = parseInt(pidInput.value);
    const adresse = parseInt(addrInput.value, 16); // On convertit l'Hexa en nombre

    if (isNaN(pid) || isNaN(adresse)) {
        resultDiv.innerHTML = "⚠️ PID ou Adresse invalide";
        return;
    }

    try {
        const valeur = await invoke('lire_valeur_memoire', { 
            pid: pid, 
            adresse: adresse 
        });
        resultDiv.innerHTML = `Valeur : <span style="color:#4ade80">${valeur}</span>`;
    } catch (err) {
        resultDiv.innerHTML = `<span style="color:red">Erreur : ${err}</span>`;
    }
}