#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pyo3::prelude::*;
use std::env;
use sysinfo::System; 

// --- COMMANDE 1 : L'IA ---
#[tauri::command] // On a enlevé le rename qui posait problème
fn envoyer_a_ia(prompt: String) -> Result<String, String> {
    unsafe {
        env::set_var("PYTHONPATH", "./core");
    }

    Python::with_gil(|py| {
        let ia_module = py.import_bound("ia_logic")
            .map_err(|e| format!("Erreur import : {}", e))?;
        
        let reponse: String = ia_module
            .getattr("demander_a_ia")
            .map_err(|e| format!("Fonction introuvable : {}", e))?
            .call1((prompt,))
            .map_err(|e| format!("Erreur appel : {}", e))?
            .extract()
            .map_err(|e| format!("Erreur extraction : {}", e))?;

        Ok(reponse)
    })
}

// --- COMMANDE 2 : LES PROCESSUS ---
#[tauri::command] // On a enlevé le rename ici aussi
fn lister_processus() -> Vec<String> {
    let mut sys = System::new_all();
    sys.refresh_all(); 

    let mut liste = Vec::new();
    for (pid, process) in sys.processes() {
      liste.push(format!("[{}] {}", pid, process.name()));
    }
    
    liste.sort(); 
    liste
}

#[tauri::command]
fn tuer_processus(pid: u32) -> bool {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    // On cherche le processus par son PID
    if let Some(process) = sys.process(sysinfo::Pid::from(pid as usize)) {
        return process.kill(); // Renvoie true si ça a réussi
    }
    false
}


fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![envoyer_a_ia, lister_processus])
        .run(tauri::generate_context!())
        .expect("Erreur au lancement de Berserker");
}

