// On stocke les survivantes localement pour l'IA
let adressesSurvivantes = [];

// --- 1. FONCTION NEXT SCAN (Filtrage Multi-types) ---
async function lancerNextScan() {
  const valInput = document.getElementById("valeur-scan");
  const typeSelect = document.getElementById("type-data"); // Récupère le type (i32/f32)
  const res = document.getElementById("res-scan");

  if (!window.targetPid) return alert("Sélectionnez un processus d'abord !");

  const valStr = valInput.value;
  const type = typeSelect.value;

  if (!valStr) return alert("Entrez la nouvelle valeur à filtrer.");

  res.innerHTML = `<i style="color:#00ff41">Filtrage ${type} en cours...</i>`;

  try {
    // Appel à Rust : On envoie la valeur en String et le type choisi
    const resultatsFiltres = await window.invoke("next_scan", {
      pid: window.targetPid,
      nouvelleValeur: valStr,
      typeData: type
    });

    // On mémorise pour le diagnostic IA
    adressesSurvivantes = resultatsFiltres || [];

    // On affiche le résultat avec les colonnes Adresse | Valeur
    afficherResultatsNext(adressesSurvivantes);
    
  } catch (err) {
    res.innerHTML = `<span style="color:red">⚠️ Erreur Next Scan : ${err}</span>`;
    console.error("Détails Next Scan:", err);
  }
}

// --- 2. AFFICHAGE DES RÉSULTATS ---
function afficherResultatsNext(adresses) {
  const res = document.getElementById("res-scan");
  const aiBtn = document.getElementById("ai-analyzer");

  if (!adresses || adresses.length === 0) {
    res.innerHTML = `Aucun résultat correspondant à la nouvelle valeur.`;
    if (aiBtn) aiBtn.classList.remove("view_btn_ia");
    return;
  }

  // Rendu Adresse | Valeur
  res.innerHTML =
    `<b style="color:#00ff41">${adresses.length}</b> adresses filtrées :<br>` +
    adresses
      .map((r) => {
        const hex = `0x${r.adresse.toString(16).toUpperCase()}`;
        return `
          <div class="addr-item" onclick="modifierValeur(${r.adresse}, '${hex}')">
            <span class="hex-addr">${hex}</span>
            <span class="val-sep">|</span>
            <span class="val-num">${r.valeur}</span>
            <span class="stickers">MODIF</span>
          </div>`;
      })
      .join("");

  // Affichage du bouton IA si le nombre de cibles est faible (diagnostic précis)
  if (aiBtn) {
    if (adresses.length > 0 && adresses.length <= 15) {
      aiBtn.classList.add("view_btn_ia");
    } else {
      aiBtn.classList.remove("view_btn_ia");
    }
  }
}

// --- 3. DIAGNOSTIC IA (Voisinage Mémoire) ---
async function analyserResultatsNextScan() {
  if (adressesSurvivantes.length === 0) return;

  const res = document.getElementById("res-scan");
  const contenuActuel = res.innerHTML; 

  res.innerHTML = contenuActuel + `<div style="color:#3b82f6; margin-top:10px;">Berserker A.I analyse les ${adressesSurvivantes.length} adresses cibles...</div>`;

  let rapport = "RAPPORT DE FILTRAGE MÉMOIRE :\n";

  for (let addr of adressesSurvivantes) {
    const hex = `0x${addr.toString(16).toUpperCase()}`;
    try {
      // On utilise window.targetPid et window.invoke pour la cohérence
      const voisinage = await window.invoke("dump_contexte_memoire", {
        pid: window.targetPid,
        adresse: addr,
      });
      rapport += `- Adresse ${hex} | Voisins : [${voisinage.join(", ")}]\n`;
    } catch (e) {
      rapport += `- Adresse ${hex} | Lecture impossible\n`;
    }
  }

  const promptIA = `${rapport}
En tant qu'expert Berserker, analyse ces adresses qui ont survécu au filtrage. 
Laquelle de ces adresses est la véritable valeur de ressource basée sur les valeurs voisines ?`;

  const input = document.getElementById("msg-input");
  if (input) {
    input.value = promptIA;
    window.envoyer(); // Déclenche ta fonction globale d'envoi du chat
  }
}

// Exposition globale pour les boutons HTML
window.lancerNextScan = lancerNextScan;
window.analyserResultatsNextScan = analyserResultatsNextScan;

