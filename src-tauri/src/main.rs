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
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, MEM_RESERVE,
    PAGE_READWRITE,
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

// --- 1. PREMIER SCAN MULTI-TYPES (ANTI-FREEZE) ---
#[tauri::command]
fn premier_scan(window: tauri::Window, pid: u32, valeur_str: String, type_data: String) {
    tauri::async_runtime::spawn(async move {
        let mut resultats = Vec::new();
        let max_addr: usize = 0x7FFFFFFFFFFF;
        let mut loop_count = 0;

        unsafe {
            if let Ok(handle) = OpenProcess(PROCESS_ALL_ACCESS, false, pid) {
                let mut base_addr = 0;
                let mut mem_info = MEMORY_BASIC_INFORMATION::default();

                while VirtualQueryEx(handle, Some(base_addr as *const _), &mut mem_info, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) != 0 {
                    if mem_info.State == MEM_COMMIT && mem_info.Protect == PAGE_READWRITE {
                        let mut buffer = vec![0u8; mem_info.RegionSize];
                        let mut bytes_read = 0;

                        if ReadProcessMemory(handle, mem_info.BaseAddress, buffer.as_mut_ptr() as *mut _, mem_info.RegionSize, Some(&mut bytes_read)).is_ok() {
                            for i in (0..(bytes_read.saturating_sub(4))).step_by(4) {
                                
                                // LOGIQUE DE DETECTION DE TYPE
                                let match_found = match type_data.as_str() {
                                    "f32" => {
                                        let target = valeur_str.parse::<f32>().unwrap_or(0.0);
                                        let current = f32::from_ne_bytes([buffer[i], buffer[i+1], buffer[i+2], buffer[i+3]]);
                                        (current - target).abs() < 0.01 // Tolérance pour les floats
                                    },
                                    _ => { // Par défaut i32
                                        let target = valeur_str.parse::<i32>().unwrap_or(0);
                                        let current = i32::from_ne_bytes([buffer[i], buffer[i+1], buffer[i+2], buffer[i+3]]);
                                        current == target
                                    }
                                };

                                if match_found {
                                    resultats.push(ResultatScan {
                                        adresse: mem_info.BaseAddress as usize + i,
                                        valeur: valeur_str.clone(),
                                    });
                                }
                                if resultats.len() >= 2000 { break; }
                            }
                        }
                        
                        loop_count += 1;
                        if loop_count % 50 == 0 {
                            let pourcentage = ((base_addr as f64 / max_addr as f64) * 100.0) as u32;
                            let _ = window.emit("scan-progress", std::cmp::min(pourcentage, 99));
                        }
                    }
                    if resultats.len() >= 2000 { break; }
                    base_addr = mem_info.BaseAddress as usize + mem_info.RegionSize;
                }
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
            let mut buffer = [0u8; 4];
            if ReadProcessMemory(handle, addr as *const _, buffer.as_mut_ptr() as *mut _, 4, None).is_ok() {
                let match_found = match type_data.as_str() {
                    "f32" => {
                        let target = nouvelle_valeur.parse::<f32>().unwrap_or(0.0);
                        let current = f32::from_ne_bytes(buffer);
                        (current - target).abs() < 0.01
                    },
                    _ => i32::from_ne_bytes(buffer) == nouvelle_valeur.parse::<i32>().unwrap_or(0),
                };

                if match_found {
                    resultats_filtres.push(ResultatScan {
                        adresse: addr,
                        valeur: nouvelle_valeur.clone(),
                    });
                }
            }
        }
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
            premier_scan, next_scan, injecter_dll
        ])
        .run(tauri::generate_context!())
        .expect("Erreur au lancement");
}

