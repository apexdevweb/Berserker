async function lancerNextScan() {
  const valInput = document.getElementById("valeur-scan");
  const res = document.getElementById("res-scan");

  // On s'assure que la valeur est un nombre
  const val = parseInt(valInput.value);

  if (isNaN(val)) {
    res.innerHTML = "Entrez un nombre valide.";
    return;
  }

  if (!targetPid) {
    res.innerHTML = "Cible non sélectionnée.";
    return;
  }

  res.innerHTML = "Filtrage des adresses...";

  try {
    // CORRECTION : Tauri v2 attend 'nouvelleValeur' (sans underscore, R majuscule)
    // car il transforme 'nouvelle_valeur' du Rust automatiquement.
    const adresses = await invoke("next_scan", {
      pid: targetPid,
      nouvelleValeur: val,
    });

    if (!adresses || adresses.length === 0) {
      res.innerHTML = "Plus aucun résultat correspondant.";
    } else {
      // Mise à jour de la liste avec les survivants
      res.innerHTML =
        `Restant : ${adresses.length} adresses<br>` +
        adresses
          .map((a) => {
            const hex = `0x${a.toString(16).toUpperCase()}`;
            // On garde la fonction modifierAdresse que tu as déjà
            return `<div onclick="modifierAdresse(${a}, '${hex}')" style="cursor:pointer; color:#10b981; padding: 2px 0;">${hex} <span class="stickers">MODIF</span></div>`;
          })
          .join("");

      if (adresses.length === 1) {
        res.innerHTML +=
          "<br><b style='color:#facc15'>✅ Adresse unique trouvée !</b>";
      }
    }
  } catch (err) {
    // Affiche l'erreur précise (ex: "missing required key...")
    res.innerHTML = `<span style="color:red">⚠️ Erreur : ${err}</span>`;
    console.error("Détails Next Scan :", err);
  }
}
