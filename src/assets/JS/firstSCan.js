// On utilise window pour partager entre les 9 scripts
window.invoke = window.__TAURI__.core.invoke;
window.listen = window.__TAURI__.event.listen;

let dernieresAdresses = []; 

// --- 1. PROGRESSION ---
listen("scan-progress", (event) => {
  const pourcentage = event.payload;
  const bar = document.getElementById("scan-bar");
  const meter = document.querySelector(".prog__meter");

  if (bar) bar.value = pourcentage;
  if (meter) {
    meter.innerText = pourcentage + "%";
    meter.style.color = pourcentage === 100 ? "#00ff41" : "#3b82f6";
  }
});

// --- 2. RÉCEPTION DES RÉSULTATS (Scan Complet) ---
listen("scan-complete", (event) => {
  const resultats = event.payload; 
  const res = document.getElementById("res-scan");
  const meter = document.querySelector(".prog__meter");

  if (meter) meter.innerText = "COMPLETE";
  dernieresAdresses = resultats; // Mémorise pour le diagnostic IA

  if (!resultats || resultats.length === 0) {
    res.innerHTML = `<span style="color:#ff4444">❌ Aucun résultat trouvé.</span>`;
  } else {
    // Affiche Adresse | Valeur (supporte Int et Float)
    res.innerHTML =
      `<b style="color:#00ff41">${resultats.length}</b> cibles identifiées :<br>` +
      resultats
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
  }
});

// --- 3. MODIFICATION ---
async function modifierValeur(addrInt, addrHex) {
  const type = document.getElementById("type-data").value; // Récupère le type actuel
  const nouvelleVal = prompt(`[BERSERKER] Modifier ${addrHex} (${type}) :`, "999999");
  
  if (!nouvelleVal) return;

  try {
    const msg = await window.invoke("ecrire_valeur_memoire", { 
        pid: window.targetPid, 
        adresse: addrInt, 
        nouvelleValeur: nouvelleVal, // On envoie en String
        typeData: type
    });
    console.log("Berserker Write:", msg);
  } catch (err) { alert("Erreur d'écriture : " + err); }
}

// --- 4. PREMIER SCAN MULTI-TYPES ---
async function lancerPremierScan() {
  const valInput = document.getElementById("valeur-scan");
  const typeSelect = document.getElementById("type-data"); // Nouveau menu déroulant
  const res = document.getElementById("res-scan");
  const bar = document.getElementById("scan-bar");

  if (!window.targetPid) return alert("⚠️ Cible manquante ! Sélectionnez un processus.");
  if (!valInput.value) return alert("⚠️ Entrez une valeur à rechercher.");

  const valStr = valInput.value;
  const type = typeSelect.value; // "i32" ou "f32"

  if (bar) bar.value = 0;
  res.innerHTML = `<i style="color:#3b82f6" class="scanning-text">🔍 Scan ${type} en cours...</i>`;

  try {
      // On envoie les arguments au nouveau moteur Rust
      await window.invoke("premier_scan", { 
          pid: window.targetPid, 
          valeurStr: valStr, 
          typeData: type 
      });
  } catch (err) {
      console.error("Erreur invoke premier_scan:", err);
      res.innerHTML = `<span style="color:red">Erreur : ${err}</span>`;
  }
}

// --- 🚀 EXPOSITION GLOBALE ---
window.lancerPremierScan = lancerPremierScan;
window.modifierValeur = modifierValeur;


