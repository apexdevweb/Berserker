// Variable globale pour stocker le processus cible (Target)
let targetPid = null;

/**
 * Filtre les processus pour ne garder que les "Applications"
 * On exclut les processus système Windows classiques qui polluent la liste.
 */
function filtrerApplications(procs) {
    const exclusions = [
        "svchost.exe", "conhost.exe", "runtimebroker.exe", "taskhostw.exe",
        "taskmgr.exe", "sihost.exe", "searchhost.exe", "startmenuexperiencehost.exe",
        "dllhost.exe", "fontdrvhost.exe", "dwm.exe", "ctfmon.exe", "lsass.exe",
        "services.exe", "wininit.exe", "winlogon.exe", "smss.exe"
    ];

    return procs.filter(p => {
        const name = p.name.toLowerCase();
        // On garde seulement si le nom n'est pas dans la liste d'exclusion
        return !exclusions.includes(name);
    });
}

async function chargerProcessus() {
    const container = document.getElementById('liste-proc');
    
    // 1. Vérification du moteur Tauri
    if (!window.__TAURI__ || !window.__TAURI__.core) {
        container.innerHTML = "Erreur : Moteur Tauri non prêt.";
        return;
    }

    const { invoke } = window.__TAURI__.core;
    container.innerHTML = "<i>Analyse des applications en cours...</i>";
    
    try {
        // 2. Appel de la commande Rust
        let procs = await invoke('lister_processus_objets');
        
        // 3. Application du filtre "Apps uniquement"
        const apps = filtrerApplications(procs);

        if (apps.length === 0) {
            container.innerHTML = "Aucune application détectée.";
            return;
        }

        // 4. Construction de la liste propre
        container.innerHTML = apps.map(p => `
            <div class="proc-item" style="display:flex; justify-content:space-between; align-items:center; padding:6px; border-bottom:1px solid #333; cursor:pointer;">
                <span onclick="selectionnerTarget(${p.pid}, '${p.name}')" style="flex:1; font-weight:500;">
                    <b style="color:#3b82f6;">[${p.pid}]</b> ${p.name}
                </span>
                <button onclick="tuer(${p.pid}, '${p.name}')" class="stickers">KILL</button>
            </div>
        `).join('');

    } catch (err) {
        container.innerHTML = `<span style="color:red">Erreur : ${err}</span>`;
        console.error("Détails :", err);
    }
}

// Fonction pour sélectionner une cible (Target)
function selectionnerTarget(pid, name) {
    targetPid = pid;
    
    // Mise à jour visuelle du bandeau Target
    const targetDisplay = document.getElementById('target-display');
    if (targetDisplay) {
        targetDisplay.innerHTML = `Target: <span style="color:#4ade80">${name}</span> <small>(${pid})</small>`;
        targetDisplay.style.background = "#1e293b"; // Petit flash visuel
    }

    // Auto-remplissage du champ PID dans le module de Scan Mémoire
    const scanPidInput = document.getElementById('scan-pid');
    if (scanPidInput) {
        scanPidInput.value = pid;
    }

    console.log(`[Berserker] Cible verrouillée : ${name} (${pid})`);
}

async function tuer(pid, name) {
    // Double vérification car "tuer" est une action critique
    const confirmation = confirm(`BERSERKER : Confirmer la destruction du processus ${name} ?`);
    
    if (confirmation) {
        try {
            const { invoke } = window.__TAURI__.core;
            const success = await invoke('tuer_processus', { pid: parseInt(pid) });
            
            if (success) {
                chargerProcessus(); // On rafraîchit la liste automatiquement
            } else {
                alert("Erreur : Impossible d'arrêter ce processus (Privilèges insuffisants).");
            }
        } catch (err) {
            alert("Erreur technique lors du kill : " + err);
        }
    }
}

function arreterScan() {
    document.getElementById('liste-proc').innerHTML = "Scan des processus arrêté.";
}

