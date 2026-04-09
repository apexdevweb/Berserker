#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use once_cell::sync::Lazy;
use pyo3::prelude::*;
use serde::Serialize;
use std::env;
use std::ffi::{c_void, CString};
use std::process::Command;
use std::sync::Mutex;
use sysinfo::System;
use tauri::Emitter;

// --- IMPORTATIONS WINDOWS NETTOYÉES ---
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
// --- MODIFIE TES IMPORTS COMME CECI ---
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, MEM_RESERVE,
    PAGE_READWRITE, PAGE_EXECUTE_READWRITE, // <--- AJOUTE CETTE LIGNE
};

use windows::Win32::System::Threading::{
    CreateRemoteThread, OpenProcess, LPTHREAD_START_ROUTINE, PROCESS_ALL_ACCESS,
};
static LISTE_ADRESSES: Lazy<Mutex<Vec<usize>>> = Lazy::new(|| Mutex::new(Vec::new()));

// Structure flexible pour envoyer des résultats hétérogènes (Int ou Float) au JS
#[derive(Serialize, Clone)]
pub struct ResultatScan {
    adresse: usize,
    valeur: String, 
}

#[derive(Serialize, Clone)]
pub struct ProcessInfo {
    pid: u32,
    name: String,
}
#[tauri::command]
fn nouveau_scan() {
    // Vide simplement le Mutex qui contient les adresses trouvées
    if let Ok(mut guard) = LISTE_ADRESSES.lock() {
        guard.clear();
    }
}
// --- 1. PREMIER SCAN MULTI-TYPES (ANTI-FREEZE) ---
#[tauri::command]
fn premier_scan(window: tauri::Window, pid: u32, valeur_str: String, type_data: String, step: usize) {
    tauri::async_runtime::spawn(async move {
        let mut resultats = Vec::new();
        let type_size = match type_data.as_str() {
            "i16" => 2,
            "i64" | "f64" => 8,
            _ => 4,
        };

        unsafe {
            if let Ok(handle) = OpenProcess(PROCESS_ALL_ACCESS, false, pid) {
                // --- ÉTAPE 1 : CALCUL DU POIDS RÉEL DE LA MÉMOIRE (POUR LA FLUIDITÉ) ---
                let mut total_memory_to_scan = 0;
                let mut temp_addr = 0;
                let mut info = MEMORY_BASIC_INFORMATION::default();
                
                while VirtualQueryEx(handle, Some(temp_addr as *const _), &mut info, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) != 0 {
                    if info.State == MEM_COMMIT && (info.Protect == PAGE_READWRITE || info.Protect == PAGE_EXECUTE_READWRITE) {
                        total_memory_to_scan += info.RegionSize;
                    }
                    temp_addr = info.BaseAddress as usize + info.RegionSize;
                }

                // --- ÉTAPE 2 : SCAN RÉEL ---
                let mut scanned_so_far = 0;
                let mut base_addr = 0;
                let mut mem_info = MEMORY_BASIC_INFORMATION::default();

                while VirtualQueryEx(handle, Some(base_addr as *const _), &mut mem_info, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) != 0 {
                    let is_writable = mem_info.State == MEM_COMMIT && 
                        (mem_info.Protect == PAGE_READWRITE || mem_info.Protect == PAGE_EXECUTE_READWRITE);

                    if is_writable {
                        let mut buffer = vec![0u8; mem_info.RegionSize];
                        let mut bytes_read = 0;

                        if ReadProcessMemory(handle, mem_info.BaseAddress, buffer.as_mut_ptr() as *mut _, mem_info.RegionSize, Some(&mut bytes_read)).is_ok() {
                            let data = &buffer[..bytes_read];

                            for i in (0..data.len().saturating_sub(type_size)).step_by(step) {
                                let chunk = &data[i..i + type_size];
                                
                                let match_found = match type_data.as_str() {
                                    "i16" => i16::from_le_bytes(chunk.try_into().unwrap_or([0;2])) == valeur_str.parse::<i16>().unwrap_or(0),
                                    "i32" => i32::from_le_bytes(chunk.try_into().unwrap_or([0;4])) == valeur_str.parse::<i32>().unwrap_or(0),
                                    "i64" => i64::from_le_bytes(chunk.try_into().unwrap_or([0;8])) == valeur_str.parse::<i64>().unwrap_or(0),
                                    "f32" => (f32::from_le_bytes(chunk.try_into().unwrap_or([0;4])) - valeur_str.parse::<f32>().unwrap_or(0.0)).abs() < 0.01,
                                    "f64" => (f64::from_le_bytes(chunk.try_into().unwrap_or([0;8])) - valeur_str.parse::<f64>().unwrap_or(0.0)).abs() < 0.001,
                                    _ => false
                                };

                                if match_found {
                                    resultats.push(ResultatScan {
                                        adresse: mem_info.BaseAddress as usize + i,
                                        valeur: valeur_str.clone(),
                                    });
                                }
                                if resultats.len() >= 5000 { break; }
                            }
                        }
                        
                        // Mise à jour de la progression basée sur les Mo réels traités
                        scanned_so_far += mem_info.RegionSize;
                        if total_memory_to_scan > 0 {
                            let progression = ((scanned_so_far as f64 / total_memory_to_scan as f64) * 100.0) as u32;
                            let _ = window.emit("scan-progress", progression.min(99));
                        }
                    }
                    
                    if resultats.len() >= 5000 { break; }
                    base_addr = mem_info.BaseAddress as usize + mem_info.RegionSize;
                }
                let _ = windows::Win32::Foundation::CloseHandle(handle);
            }
        }

        if let Ok(mut guard) = LISTE_ADRESSES.lock() {
            *guard = resultats.iter().map(|r| r.adresse).collect();
        }

        let _ = window.emit("scan-progress", 100);
        let _ = window.emit("scan-complete", resultats);
    });
}



// --- 2. NEXT SCAN MULTI-TYPES ---
#[tauri::command]
fn next_scan(pid: u32, nouvelle_valeur: String, type_data: String) -> Result<Vec<ResultatScan>, String> {
    let mut adresses_globales = LISTE_ADRESSES.lock().unwrap();
    let mut resultats_filtres = Vec::new();

    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid).map_err(|_| "Admin requis !")?;

        for &addr in adresses_globales.iter() {
            // Déterminer la taille à lire selon le type
            let size = match type_data.as_str() {
                "i16" => 2,
                "i32" | "f32" => 4,
                "i64" | "f64" => 8,
                _ => 4,
            };

            let mut buffer = vec![0u8; size];
            if ReadProcessMemory(handle, addr as *const _, buffer.as_mut_ptr() as *mut _, size, None).is_ok() {
                
                let match_found = match type_data.as_str() {
                    "i16" => i16::from_le_bytes(buffer[..2].try_into().unwrap_or([0;2])) == nouvelle_valeur.parse::<i16>().unwrap_or(0),
                    "i32" => i32::from_le_bytes(buffer[..4].try_into().unwrap_or([0;4])) == nouvelle_valeur.parse::<i32>().unwrap_or(0),
                    "i64" => i64::from_le_bytes(buffer[..8].try_into().unwrap_or([0;8])) == nouvelle_valeur.parse::<i64>().unwrap_or(0),
                    "f32" => {
                        let target = nouvelle_valeur.parse::<f32>().unwrap_or(0.0);
                        let current = f32::from_le_bytes(buffer[..4].try_into().unwrap_or([0;4]));
                        (current - target).abs() < 0.01
                    },
                    "f64" => {
                        let target = nouvelle_valeur.parse::<f64>().unwrap_or(0.0);
                        let current = f64::from_le_bytes(buffer[..8].try_into().unwrap_or([0;8]));
                        (current - target).abs() < 0.001
                    },
                    _ => false,
                };

                if match_found {
                    resultats_filtres.push(ResultatScan {
                        adresse: addr,
                        valeur: nouvelle_valeur.clone(),
                    });
                }
            }
        }
        let _ = windows::Win32::Foundation::CloseHandle(handle);
    }
    
    *adresses_globales = resultats_filtres.iter().map(|r| r.adresse).collect();
    Ok(resultats_filtres)
}

// --- 3. ECRITURE MULTI-TYPES ---
#[tauri::command]
fn ecrire_valeur_memoire(pid: u32, adresse: usize, nouvelle_valeur: String, type_data: String) -> Result<String, String> {
    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid).map_err(|_| "Admin requis !")?;
        
        let buffer = match type_data.as_str() {
            "f32" => nouvelle_valeur.parse::<f32>().map_err(|_| "Float invalide")?.to_ne_bytes().to_vec(),
            _ => nouvelle_valeur.parse::<i32>().map_err(|_| "Int invalide")?.to_ne_bytes().to_vec(),
        };

        WriteProcessMemory(handle, adresse as *const c_void, buffer.as_ptr() as *const c_void, buffer.len(), None)
            .map_err(|_| "Échec écriture")?;
        Ok(format!("Succès : {} écrit", nouvelle_valeur))
    }
}

// --- 4. DLL INJECTOR (RÉPARÉ) ---
#[tauri::command]
fn injecter_dll(pid: u32, dll_path: String) -> Result<String, String> {
    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid).map_err(|_| "Admin requis")?;
        let path_cstr = CString::new(dll_path).map_err(|_| "Chemin invalide")?;
        let path_len = path_cstr.as_bytes_with_nul().len();

        let remote_mem = VirtualAllocEx(handle, None, path_len, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        if remote_mem.is_null() { return Err("Allocation échouée".into()); }

        let _ = WriteProcessMemory(handle, remote_mem, path_cstr.as_ptr() as *const c_void, path_len, None);

        let kernel32 = GetModuleHandleA(windows::core::s!("kernel32.dll")).map_err(|_| "Kernel32 introuvable")?;
        let load_lib_fn = GetProcAddress(kernel32, windows::core::s!("LoadLibraryA")).ok_or("LoadLibraryA introuvable")?;

        let thread_routine: LPTHREAD_START_ROUTINE = std::mem::transmute(load_lib_fn);

        CreateRemoteThread(handle, None, 0, thread_routine, Some(remote_mem), 0, None)
            .map_err(|_| "Injection échouée")?;

        Ok("💉 Injection réussie !".into())
    }
}

// --- 5. IA (AVEC GESTION D'ERREURS PYTHON) ---
#[tauri::command]
fn envoyer_a_ia(prompt: String) -> Result<String, String> {
    unsafe { env::set_var("PYTHONPATH", "./core"); }
    Python::with_gil(|py| {
        let ia = py.import_bound("ia_logic").map_err(|e| e.to_string())?;
        let res: String = ia.getattr("demander_a_ia").map_err(|e| e.to_string())?
            .call1((prompt,)).map_err(|e| e.to_string())?
            .extract().map_err(|e| e.to_string())?;
        Ok(res)
    })
}

// --- 6. SYSTEM UTILS ---
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

fn main() {
    let _ = Command::new("ollama").arg("serve").spawn();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            envoyer_a_ia, lister_processus_objets, ecrire_valeur_memoire,
            premier_scan, next_scan, nouveau_scan, injecter_dll
        ])
        .run(tauri::generate_context!())
        .expect("Erreur au lancement");
}

