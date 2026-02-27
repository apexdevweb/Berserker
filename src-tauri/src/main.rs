#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pyo3::prelude::*;
use std::env;
use std::ffi::c_void;
use std::process::Command;
use sysinfo::System;
use serde::Serialize;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Importations Windows
use windows::Win32::System::Memory::{VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_READWRITE, PAGE_READONLY};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};

// --- VARIABLE GLOBALE POUR LE SCAN ---
// On utilise Lazy et Mutex pour stocker les adresses entre First Scan et Next Scan
static LISTE_ADRESSES: Lazy<Mutex<Vec<usize>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[derive(Serialize)]
pub struct ProcessInfo {
    pid: u32,
    name: String,
}

// --- 1. PREMIER SCAN ---
#[tauri::command]
fn premier_scan(pid: u32, valeur_recherchee: i32) -> Result<Vec<usize>, String> {
    let mut adresses_trouvees = Vec::new();
    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid)
            .map_err(|_| "Accès refusé ! Lancez en ADMIN.")?;

        let mut base_addr = 0;
        let mut mem_info = MEMORY_BASIC_INFORMATION::default();

        while VirtualQueryEx(handle, Some(base_addr as *const _), &mut mem_info, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) != 0 {
            if mem_info.State == MEM_COMMIT && (mem_info.Protect == PAGE_READWRITE || mem_info.Protect == PAGE_READONLY) {
                let mut buffer = vec![0u8; mem_info.RegionSize];
                let mut bytes_read = 0;

                if ReadProcessMemory(handle, mem_info.BaseAddress, buffer.as_mut_ptr() as *mut _, mem_info.RegionSize, Some(&mut bytes_read)).is_ok() {
                    for i in 0..(bytes_read.saturating_sub(4)) {
                        let val = i32::from_ne_bytes([buffer[i], buffer[i+1], buffer[i+2], buffer[i+3]]);
                        if val == valeur_recherchee {
                            adresses_trouvees.push(mem_info.BaseAddress as usize + i);
                        }
                        if adresses_trouvees.len() >= 2000 { break; }
                    }
                }
            }
            if adresses_trouvees.len() >= 2000 { break; }
            base_addr = mem_info.BaseAddress as usize + mem_info.RegionSize;
        }
    }
    
    // CORRECTION : Assignation correcte dans le Mutex
    let mut guard = LISTE_ADRESSES.lock().unwrap();
    *guard = adresses_trouvees.clone();
    
    Ok(adresses_trouvees)
}

// --- 2. NEXT SCAN ---
#[tauri::command]
fn next_scan(pid: u32, nouvelle_valeur: i32) -> Result<Vec<usize>, String> {
    let mut adresses_globales = LISTE_ADRESSES.lock().unwrap();
    let mut resultats_filtres = Vec::new();

    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid)
            .map_err(|_| "Admin requis !")?;

        for &addr in adresses_globales.iter() {
            let mut buffer: i32 = 0;
            let mut bytes_read = 0;

            let ok = ReadProcessMemory(handle, addr as *const _, &mut buffer as *mut _ as *mut _, 4, Some(&mut bytes_read));

            if ok.is_ok() && buffer == nouvelle_valeur {
                resultats_filtres.push(addr);
            }
        }
    }

    // Mise à jour de la liste globale avec les survivants
    *adresses_globales = resultats_filtres.clone();
    Ok(resultats_filtres)
}

// --- 3. ECRITURE MÉMOIRE ---
#[tauri::command]
fn ecrire_valeur_memoire(pid: u32, adresse: usize, nouvelle_valeur: i32) -> Result<String, String> {
    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid)
            .map_err(|_| "Admin requis !")?;

        let buffer = nouvelle_valeur.to_ne_bytes();
        let mut bytes_written = 0;

        let ok = WriteProcessMemory(
            handle,
            adresse as *const c_void,
            buffer.as_ptr() as *const c_void,
            4,
            Some(&mut bytes_written),
        );

        if ok.is_ok() {
            Ok(format!("Succès : {} écrit à 0x{:X}", nouvelle_valeur, adresse))
        } else {
            Err("Échec de l'écriture (Mémoire protégée)".to_string())
        }
    }
}

// --- 4. IA BERSERKER ---
#[tauri::command]
fn envoyer_a_ia(prompt: String) -> Result<String, String> {
    unsafe { env::set_var("PYTHONPATH", "./core"); }
    Python::with_gil(|py| {
        let ia_module = py.import_bound("ia_logic").map_err(|e| e.to_string())?;
        let reponse: String = ia_module.getattr("demander_a_ia")
            .map_err(|e| e.to_string())?
            .call1((prompt,))
            .map_err(|e| e.to_string())?
            .extract()
            .map_err(|e| e.to_string())?;
        Ok(reponse)
    })
}

// --- 5. LISTER LES APPS ---
#[tauri::command]
fn lister_processus_objets() -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut apps: Vec<ProcessInfo> = sys.processes().values()
        .filter(|p| p.memory() > 5_000_000)
        .map(|p| ProcessInfo { pid: p.pid().as_u32(), name: p.name().to_string() })
        .collect();
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

// --- 6. TUER PROCESSUS ---
#[tauri::command]
fn tuer_processus(pid: u32) -> bool {
    let mut sys = System::new_all();
    sys.refresh_all();
    if let Some(process) = sys.process(sysinfo::Pid::from(pid as usize)) {
        return process.kill();
    }
    false
}

// --- 7. RELANCER OLLAMA ---
#[tauri::command]
fn relancer_ollama() -> Result<String, String> {
    let _ = Command::new("taskkill").args(["/F", "/IM", "ollama.exe"]).output();
    Command::new("ollama").arg("serve").spawn().map_err(|e| e.to_string())?;
    Ok("Ollama relancé".to_string())
}

fn main() {
    let _ = Command::new("ollama").arg("serve").spawn();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            envoyer_a_ia, 
            lister_processus_objets, 
            ecrire_valeur_memoire,
            premier_scan,
            next_scan,
            tuer_processus,
            relancer_ollama
        ])
        .run(tauri::generate_context!())
        .expect("Erreur au lancement");
}
