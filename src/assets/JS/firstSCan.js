const { listen } = window.__TAURI__.event;

// --- 1. PROGRESSION (Barre et Compteur) ---
listen("scan-progress", (event) => {
  const pourcentage = event.payload;
  const bar = document.getElementById("scan-bar");
  const meter = document.getElementById("progress-meter");

  if (bar) bar.value = pourcentage;
  if (meter) {
    meter.innerText = pourcentage + "%";
    // Utilisation de tes couleurs de dégradé via JS si besoin
    meter.style.color = pourcentage === 100 ? "#00ff41" : "#3b82f6";
  }
});
let dernieresAdresses = []; // On mémorise les résultats ici
// --- 2. RÉCEPTION DES RÉSULTATS (Fin du scan threadé) ---
listen("scan-complete", (event) => {
  dernieresAdresses = event.payload;
  const adresses = event.payload;
  const res = document.getElementById("res-scan");

  if (!adresses || adresses.length === 0) {
    res.innerHTML = `Aucun résultat trouvé.`;
  } else {
    // On génère la liste des adresses cliquables
    res.innerHTML =
      `<b style="color:#00ff41">${adresses.length}</b> adresses infiltrées :<br>` +
      adresses
        .map((a) => {
          const hex = `0x${a.toString(16).toUpperCase()}`;
          return `<div class="addr-item" onclick="modifierValeur(${a}, '${hex}')">${hex} <span class="stickers">MODIF</span></div>`;
        })
        .join("");
  }
});

// --- FONCTION DE DIAGNOSTIC ---
async function analyserParIA() {
  if (dernieresAdresses.length === 0) return alert("Fais un scan d'abord !");
  
  const resIA = document.getElementById("res-scan");
  resIA.innerHTML = "L'IA analyse le voisinage de la mémoire...";

  let rapportComplet = "ANALYSE DE CONTEXTE MÉMOIRE :\n";

  for (let addr of dernieresAdresses) {
    const hex = `0x${addr.toString(16).toUpperCase()}`;
    try {
      // On demande à Rust le voisinage de l'adresse
      const voisinage = await invoke('dump_contexte_memoire', { pid: targetPid, adresse: addr });
      rapportComplet += `- Adresse ${hex} | Voisinage : [${voisinage.join(', ')}]\n`;
    } catch (e) {
      rapportComplet += `- Adresse ${hex} | Inaccessible\n`;
    }
  }

  const promptIA = `${rapportComplet}
  En tant qu'expert Gate Crasher, analyse ces données de Dynasty Warriors. 
  Cherche des structures de données (valeurs qui se suivent). 
  Laquelle de ces adresses est le véritable 'Gold Pointer' ?`;

  // On injecte dans le chat
  const input = document.getElementById("msg-input");
  input.value = promptIA;
  envoyer(); 
}


// --- 3. MODIFICATION (Write Memory) ---
async function modifierValeur(addrInt, addrHex) {
  const nouvelleVal = prompt(`[BERSERKER] Modifier ${addrHex} :`, "999999");
  if (!nouvelleVal) return;

  try {
    const msg = await invoke("ecrire_valeur_memoire", {
      pid: targetPid,
      adresse: addrInt,
      nouvelleValeur: parseInt(nouvelleVal),
    });
    console.log("Berserker Write:", msg);
  } catch (err) {
    alert("Erreur d'écriture : " + err);
  }
}

// --- 4. LANCEMENT (Zéro Freeze) ---
async function lancerPremierScan() {
  const res = document.getElementById("res-scan");
  const bar = document.getElementById("scan-bar");
  const valInput = document.getElementById("valeur-scan");

  if (!targetPid) return alert("Cible manquante ! Sélectionnez un processus.");

  const val = parseInt(valInput.value);
  if (isNaN(val)) return alert("Veuillez entrer un nombre.");

  // Reset visuel
  if (bar) bar.value = 0;
  res.innerHTML = `<i style="color:#3b82f6">Infiltration de la RAM en cours...</i>`;

  try {
    // On appelle Rust : il lance le thread et rend la main au JS en 1ms. [1.2]
    await invoke("premier_scan", {
      pid: targetPid,
      valeurRecherchee: val,
    });
  } catch (err) {
    res.innerHTML = `<span style="color:red">⚠️ Erreur de lancement : ${err}</span>`;
  }
}
