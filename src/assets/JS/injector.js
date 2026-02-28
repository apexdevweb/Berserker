const { open } = window.__TAURI__.dialog;

async function choisirEtInjecter() {
    // 1. Ouvre la boîte de dialogue Windows pour choisir la DLL
    const selected = await open({
        multiple: false,
        filters: [{
            name: 'Dynamic Link Library',
            extensions: ['dll']
        }]
    });

    if (selected && targetPid) {
        const resLog = document.getElementById('res-scan');
        resLog.innerHTML = `📡 Injection de : ${selected.split('\\').pop()}...`;

        try {
            // 2. Envoie le chemin récupéré à Rust
            const msg = await invoke('injecter_dll', { 
                pid: targetPid, 
                dllPath: selected 
            });
            resLog.innerHTML = `<b style="color:#00ff41">${msg}</b>`;
        } catch (err) {
            resLog.innerHTML = `<b style="color:#ff4444">❌ Erreur : ${err}</b>`;
        }
    }
}
