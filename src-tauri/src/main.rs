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

#[derive(Serialize, Clone)]
pub struct ProcessInfo {
    pid: u32,
    name: String,
}

// --- 1. PREMIER SCAN (ASYNCHRONE) ---
#[tauri::command]
fn premier_scan(window: tauri::Window, pid: u32, valeur_recherchee: i32) {
    tauri::async_runtime::spawn(async move {
        let mut adresses_trouvees = Vec::new();
        let max_addr: usize = 0x7FFFFFFFFFFF;

        unsafe {
            if let Ok(handle) = OpenProcess(PROCESS_ALL_ACCESS, false, pid) {
                let mut base_addr = 0;
                let mut mem_info = MEMORY_BASIC_INFORMATION::default();

                while VirtualQueryEx(
                    handle,
                    Some(base_addr as *const _),
                    &mut mem_info,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                ) != 0
                {
                    if mem_info.State == MEM_COMMIT && mem_info.Protect == PAGE_READWRITE {
                        let mut buffer = vec![0u8; mem_info.RegionSize];
                        let mut bytes_read = 0;

                        if ReadProcessMemory(
                            handle,
                            mem_info.BaseAddress,
                            buffer.as_mut_ptr() as *mut _,
                            mem_info.RegionSize,
                            Some(&mut bytes_read),
                        )
                        .is_ok()
                        {
                            for i in (0..(bytes_read.saturating_sub(4))).step_by(4) {
                                let val = i32::from_ne_bytes([
                                    buffer[i],
                                    buffer[i + 1],
                                    buffer[i + 2],
                                    buffer[i + 3],
                                ]);
                                if val == valeur_recherchee {
                                    adresses_trouvees.push(mem_info.BaseAddress as usize + i);
                                }
                                if adresses_trouvees.len() >= 2000 {
                                    break;
                                }
                            }
                        }
                        let pourcentage = ((base_addr as f64 / max_addr as f64) * 100.0) as u32;
                        let _ = window.emit("scan-progress", pourcentage);
                    }
                    if adresses_trouvees.len() >= 2000 {
                        break;
                    }
                    base_addr = mem_info.BaseAddress as usize + mem_info.RegionSize;
                }
            }
        }
        if let Ok(mut guard) = LISTE_ADRESSES.lock() {
            *guard = adresses_trouvees.clone();
        }
        let _ = window.emit("scan-progress", 100);
        let _ = window.emit("scan-complete", adresses_trouvees);
    });
}

// --- 2. DUMP CONTEXTE ---
#[tauri::command]
fn dump_contexte_memoire(pid: u32, adresse: usize) -> Result<Vec<i32>, String> {
    let mut contexte = Vec::new();
    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid).map_err(|_| "Admin requis")?;
        let start_addr = adresse.saturating_sub(16);
        for i in 0..12 {
            let current_addr = start_addr + (i * 4);
            let mut buffer: i32 = 0;
            let ok = ReadProcessMemory(
                handle,
                current_addr as *const _,
                &mut buffer as *mut _ as *mut _,
                4,
                None,
            );
            contexte.push(if ok.is_ok() { buffer } else { -1 });
        }
    }
    Ok(contexte)
}

// --- 3. NEXT SCAN ---
#[tauri::command]
fn next_scan(pid: u32, nouvelle_valeur: i32) -> Result<Vec<usize>, String> {
    let mut adresses_globales = LISTE_ADRESSES.lock().unwrap();
    let mut resultats_filtres = Vec::new();
    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid).map_err(|_| "Admin requis !")?;
        for &addr in adresses_globales.iter() {
            let mut buffer: i32 = 0;
            let _ = ReadProcessMemory(
                handle,
                addr as *const _,
                &mut buffer as *mut _ as *mut _,
                4,
                None,
            );
            if buffer == nouvelle_valeur {
                resultats_filtres.push(addr);
            }
        }
    }
    *adresses_globales = resultats_filtres.clone();
    Ok(resultats_filtres)
}

// --- 4. ECRITURE ---
#[tauri::command]
fn ecrire_valeur_memoire(pid: u32, adresse: usize, nouvelle_valeur: i32) -> Result<String, String> {
    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid).map_err(|_| "Admin requis !")?;
        let buffer = nouvelle_valeur.to_ne_bytes();
        WriteProcessMemory(
            handle,
            adresse as *const c_void,
            buffer.as_ptr() as *const c_void,
            4,
            None,
        )
        .map_err(|_| "Échec écriture")?;
        Ok(format!("Succès : {} écrit", nouvelle_valeur))
    }
}

// --- 5. DLL INJECTOR (CORRECTION TYPE) ---
#[tauri::command]
fn injecter_dll(pid: u32, dll_path: String) -> Result<String, String> {
    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid).map_err(|_| "Admin requis")?;
        let path_cstr = CString::new(dll_path).map_err(|_| "Chemin invalide")?;
        let path_len = path_cstr.as_bytes_with_nul().len();

        let remote_mem = VirtualAllocEx(
            handle,
            None,
            path_len,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );
        if remote_mem.is_null() {
            return Err("Allocation échouée".into());
        }

        WriteProcessMemory(
            handle,
            remote_mem,
            path_cstr.as_ptr() as *const c_void,
            path_len,
            None,
        )
        .map_err(|_| "Échec transfert chemin")?;

        let kernel32 = GetModuleHandleA(windows::core::s!("kernel32.dll"))
            .map_err(|_| "Kernel32 introuvable")?;

        // 1. On récupère l'adresse brute
        let load_lib_fn = GetProcAddress(kernel32, windows::core::s!("LoadLibraryA"))
            .ok_or("LoadLibraryA introuvable")?;

        // 2. On transmute ET on déballe l'Option pour avoir le type LPTHREAD_START_ROUTINE pur [1.3]
        let thread_start_routine: LPTHREAD_START_ROUTINE = std::mem::transmute(load_lib_fn);

        // 3. On passe la valeur au Thread (on utilise unwrap car CreateRemoteThread attend un type spécifique) [1.4]
        CreateRemoteThread(
            handle,
            None,
            0,
            thread_start_routine, // Plus besoin de Some() ici, CreateRemoteThread prend l'Option interne [1.5]
            Some(remote_mem),
            0,
            None,
        )
        .map_err(|_| "Thread Injection échoué")?;

        Ok("💉 DLL Injectée avec succès !".into())
    }
}

// --- 6. IA (CORRECTION PYERR) ---
#[tauri::command]
fn envoyer_a_ia(prompt: String) -> Result<String, String> {
    unsafe {
        env::set_var("PYTHONPATH", "./core");
    }
    Python::with_gil(|py| {
        let ia = py.import_bound("ia_logic").map_err(|e| e.to_string())?;
        let res: String = ia
            .getattr("demander_a_ia")
            .map_err(|e| e.to_string())?
            .call1((prompt,))
            .map_err(|e| e.to_string())?
            .extract()
            .map_err(|e| e.to_string())?;
        Ok(res)
    })
}

#[tauri::command]
fn lister_processus_objets() -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut apps: Vec<ProcessInfo> = sys
        .processes()
        .values()
        .filter(|p| p.memory() > 5_000_000)
        .map(|p| ProcessInfo {
            pid: p.pid().as_u32(),
            name: p.name().to_string(),
        })
        .collect();
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

#[tauri::command]
fn relancer_ollama() -> Result<String, String> {
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "ollama*"])
        .output();
    std::thread::sleep(std::time::Duration::from_secs(1));
    Command::new("ollama")
        .arg("serve")
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok("🛡️ Système IA relancé".to_string())
}

fn main() {
    let _ = Command::new("ollama").arg("serve").spawn();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            envoyer_a_ia,
            lister_processus_objets,
            ecrire_valeur_memoire,
            premier_scan,
            next_scan,
            dump_contexte_memoire,
            injecter_dll,
            relancer_ollama
        ])
        .run(tauri::generate_context!())
        .expect("Erreur au lancement");
}
