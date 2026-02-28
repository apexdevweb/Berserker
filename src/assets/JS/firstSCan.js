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
  const resultats = event.payload; // C'est maintenant une liste d'objets !
  const res = document.getElementById("res-scan");
  const meter = document.querySelector(".prog__meter");

  if (meter) meter.innerText = "COMPLETE";

  if (!resultats || resultats.length === 0) {
    res.innerHTML = `<span style="color:#ff4444">❌ Aucun résultat trouvé.</span>`;
  } else {
    // On génère une liste avec Adresse | Valeur
    res.innerHTML =
      `<b style="color:#00ff41">${resultats.length}</b> adresses infiltrées :<br>` +
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

// --- 3. FONCTION D'AFFICHAGE ---
function afficherResultats(adresses) {
    const res = document.getElementById("res-scan");
    if (!adresses || adresses.length === 0) {
        res.innerHTML = `❌ Aucun résultat trouvé.`;
        return;
    }
    res.innerHTML = `<b style="color:#00ff41">${adresses.length}</b> adresses :<br>` +
      adresses.map((a) => {
          const hex = `0x${a.toString(16).toUpperCase()}`;
          return `<div class="addr-item" onclick="modifierValeur(${a}, '${hex}')">${hex} <span class="stickers">MODIF</span></div>`;
      }).join("");
}

// --- 4. MODIFICATION & PREMIER SCAN ---
async function modifierValeur(addrInt, addrHex) {
  const nouvelleVal = prompt(`[BERSERKER] Modifier ${addrHex} :`, "999999");
  if (!nouvelleVal) return;
  try {
    const msg = await invoke("ecrire_valeur_memoire", { 
        pid: targetPid, 
        adresse: addrInt, 
        nouvelleValeur: parseInt(nouvelleVal) 
    });
    console.log(msg);
  } catch (err) { alert("Erreur d'écriture : " + err); }
}

async function lancerPremierScan() {
  const valInput = document.getElementById("valeur-scan");
  if (!valInput) return;
  
  const val = parseInt(valInput.value);
  if (!targetPid || isNaN(val)) return alert("Sélectionnez une cible et une valeur !");

  document.getElementById("scan-bar").value = 0;
  document.getElementById("res-scan").innerHTML = `<i style="color:#3b82f6">Infiltration RAM...</i>`;

  try {
      await invoke("premier_scan", { pid: targetPid, valeurRecherchee: val });
  } catch (err) {
      console.error("Erreur invoke premier_scan:", err);
  }
}

// --- 🚀 EXPOSITION GLOBALE (Le secret de Gate Crasher) ---
window.lancerPremierScan = lancerPremierScan;
window.modifierValeur = modifierValeur;
window.afficherResultats = afficherResultats; [1.1, 1.2]

