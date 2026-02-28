// On stocke les survivantes localement pour l'IA
let adressesSurvivantes = [];

// --- 1. FONCTION NEXT SCAN ---
async function lancerNextScan() {
  const valInput = document.getElementById("valeur-scan");
  const res = document.getElementById("res-scan");

  if (!window.targetPid) return alert("⚠️ Sélectionnez un processus d'abord !");

  const val = parseInt(valInput.value);
  if (isNaN(val)) return alert("⚠️ Entrez une valeur numérique.");

  res.innerHTML = "🔍 Filtrage chirurgical en cours...";

  try {
    // Appel à ton i9 : Rust renvoie maintenant une liste d'objets {adresse, valeur} [1.2]
    const resultatsFiltres = await window.invoke("next_scan", {
      pid: window.targetPid,
      nouvelleValeur: val,
    });

    // On mémorise pour le diagnostic IA
    adressesSurvivantes = resultatsFiltres || [];

    // On affiche le résultat avec les colonnes Adresse | Valeur [1.3]
    afficherResultatsNext(adressesSurvivantes);
    
  } catch (err) {
    res.innerHTML = `<span style="color:red">⚠️ Erreur Next Scan : ${err}</span>`;
    console.error(err);
  }
}

// --- 2. AFFICHAGE DES RÉSULTATS DU NEXT SCAN ---
function afficherResultatsNext(adresses) {
  const res = document.getElementById("res-scan");
  const aiBtn = document.getElementById("ai-analyzer");

  if (!adresses || adresses.length === 0) {
    res.innerHTML = `❌ Aucun résultat correspondant à la nouvelle valeur.`;
    if (aiBtn) aiBtn.classList.remove("view_btn_ia"); // On cache le bouton si vide [1.4]
    return;
  }

  // Rendu propre : Adresse | Valeur (comme dans le firstScan)
  res.innerHTML =
    `<b style="color:#00ff41">${adresses.length}</b> adresses restantes :<br>` +
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

  // Gestion de l'affichage du bouton IA [1.5]
  if (aiBtn) {
    if (adresses.length > 0 && adresses.length <= 15) {
      aiBtn.classList.add("view_btn_ia");
    } else {
      aiBtn.classList.remove("view_btn_ia");
    }
  }
}

// On expose la fonction pour le bouton HTML
window.lancerNextScan = lancerNextScan;


// --- 3. DIAGNOSTIC IA (Voisinage Mémoire) ---
async function analyserResultatsNextScan() {
  if (adressesSurvivantes.length === 0) return;

  const res = document.getElementById("res-scan");
  
  const contenuActuel = res.innerHTML; 

  res.innerHTML = contenuActuel + `Berserker A.I analyse sur les ${adressesSurvivantes.length} adresses cibles`;

  let rapport = "RAPPORT DE FILTRAGE MÉMOIRE :\n";

  for (let addr of adressesSurvivantes) {
    const hex = `0x${addr.toString(16).toUpperCase()}`;
    try {
      // Lecture des 12 entiers autour de l'adresse via Rust [1.5]
      const voisinage = await invoke("dump_contexte_memoire", {
        pid: targetPid,
        adresse: addr,
      });
      rapport += `- Adresse ${hex} | Voisins : [${voisinage.join(", ")}]\n`;
    } catch (e) {
      rapport += `- Adresse ${hex} | Lecture impossible\n`;
    }
  }

  const promptIA = `${rapport}
  En tant qu'expert Berserker, analyse ces adresses qui ont survécu au Next Scan. 
  Laquelle de ces adresses est le véritable valeur de ressource basé sur les valeurs voisines ?`;

  // Injection dans ton chat Ollama [1.6]
  const input = document.getElementById("msg-input");
  if (input) {
    input.value = promptIA;
    envoyer(); // Déclenche ta fonction globale d'envoi du chat
  }
}
