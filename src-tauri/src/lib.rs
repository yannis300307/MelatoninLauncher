#![allow(unused)]

use directories::ProjectDirs;
use reqwest::blocking;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

const TM_DOMAINE: &str = "https://team-melatonin.fr/";

#[cfg(target_os = "linux")]
fn get_steam_path() -> String {
    let mut steam_path = std::env::var("HOME").expect("Impossible de trouver le dossier home.");
    steam_path.push_str("/.local/share/Steam");
    if Path::exists(Path::new(&steam_path)) {
        steam_path
    } else {
        todo!("Mettre une erreur");
    }
}

#[cfg(target_os = "windows")]
fn get_steam_path() -> String {
    use {winreg::enums::*, winreg::RegKey};
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let steam_registry_info = hklm
        .open_subkey("SOFTWARE\\WOW6432Node\\Valve\\Steam")
        .expect("Can't find Steam in registry.");
    let steam_path: String = steam_registry_info
        .get_value("InstallPath")
        .expect("InstallPath is not readable.");
    steam_path
}

#[derive(Deserialize, Debug)]
struct MelatoninInfo {
    contacts: HashMap<String, Contact>,
    patches: HashMap<String, Patch>,
}

#[derive(Deserialize, Debug)]
struct Contact {
    name: String,
    role: String,
    icon: String,
    contacts: Vec<ContactSocial>,
}

#[derive(Deserialize, Debug)]
struct ContactSocial {
    name: String,
    icon: String,
    url: String,
}

#[derive(Deserialize, Debug)]
struct Patch {
    name: String,
    patch_version: String,
    steam_id: String,
    download_link: String,
    card_image: String,
    traductors: Vec<String>,
    artits: Vec<String>,
    testers: Vec<String>,
}

enum GetPatchesInfoError {
    CantReadFile(String),
    NetworkError(String),
}

#[derive(Deserialize, Debug)]
struct LibraryFolders {
    #[serde(flatten)]
    folders: HashMap<String, Library>,
}

#[derive(Deserialize, Debug)]
struct Library {
    path: PathBuf,
    apps: HashMap<String, String>,
}

impl MelatoninInfo {
    fn get_from_remote() -> Result<MelatoninInfo, GetPatchesInfoError> {
        let patches_info_response = blocking::get(format!("{}infos.json", TM_DOMAINE));

        match patches_info_response {
            Ok(data) => {
                let infos: MelatoninInfo = match data.json() {
                    Ok(data) => data,
                    Err(error) => {
                        return Err(GetPatchesInfoError::CantReadFile(format!("{}", error)))
                    }
                };

                Ok(infos)
            }
            Err(error) => Err(GetPatchesInfoError::NetworkError(format!("{}", error))),
        }
    }

    fn get_available_patches(&self) -> Vec<String> {
        let mut patches_id: Vec<String> = Vec::new();
        for patch in self.patches.values() {
            patches_id.push(patch.steam_id.to_owned());
        }

        patches_id
    }
}

#[tauri::command]
fn get_steam_installed_apps(state: tauri::State<'_, LauncherState>) -> Vec<String> {
    let mut core = state.0.lock().unwrap();

    if core.melatonin_info.is_none() {
        core.update_melatonin_info();
    }

    if let Some(melatonin_info) = &core.melatonin_info {
        for patch in melatonin_info.get_available_patches() {
            println!("{}", patch)
        }
    } else {
        println!("Can't read patches");
    }

    // Get the Steam path from the registry
    let steam_path = get_steam_path();

    let file = File::open(
        Path::new(&steam_path)
            .join("steamapps")
            .join("libraryfolders.vdf"),
    )
    .expect("Can't access library folder file.");
    let folders: LibraryFolders = keyvalues_serde::from_reader(file).unwrap();

    let mut games: Vec<String> = Vec::new();

    for folder in folders.folders.values() {
        for app in folder.apps.values() {
            games.push(app.to_owned());
        }
    }

    games
}

struct MelatoninLauncher {
    melatonin_info: Option<MelatoninInfo>,
}

impl MelatoninLauncher {
    fn new() -> MelatoninLauncher {
        MelatoninLauncher {
            melatonin_info: None,
        }
    }

    fn update_melatonin_info(&mut self) -> Result<(), GetPatchesInfoError> {
        let updated = MelatoninInfo::get_from_remote();
        match updated {
            Ok(info) => {
                self.melatonin_info = Some(info);

                Ok(())
            }
            Err(error) => Err(error),
        }
    }
}

struct LauncherState(pub Mutex<MelatoninLauncher>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(LauncherState(Mutex::new(MelatoninLauncher {
            melatonin_info: None,
        })))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_steam_installed_apps])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
