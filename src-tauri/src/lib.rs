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

#[derive(Debug)]
enum SteamPathDetectionError {
    CantReadInRegistry(String),
    CantFindSteamDirectory(String),
}

#[cfg(target_os = "linux")]
fn get_steam_path() -> Result<String, SteamPathDetectionError> {
    let mut steam_path = match std::env::var("HOME") {
        Ok(data) => data,
        Err(error) => {
            return Err(SteamPathDetectionError::CantFindSteamDirectory(
                "Impossible de trouver le dossier home.".to_string(),
            ))
        }
    };
    steam_path.push_str("/.local/share/Steam");
    if Path::exists(Path::new(&steam_path)) {
        Ok(steam_path)
    } else {
        return Err(SteamPathDetectionError::CantFindSteamDirectory(
            "Impssible de détecter Steam.".to_string(),
        ));
    }
}

#[cfg(target_os = "windows")]
fn get_steam_path() -> Result<String, SteamPathDetectionError> {
    use {
        std::f64::consts::E,
        winreg::{enums::*, RegKey},
    };
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let steam_registry_info = match hklm.open_subkey("SOFTWARE\\WOW6432Node\\Valve\\Steam") {
        Ok(data) => data,
        Err(_error) => {
            return Err(SteamPathDetectionError::CantReadInRegistry(
                "Impossible de trouver Steam dans le registre.".to_string(),
            ))
        }
    };
    let steam_path: String = match steam_registry_info.get_value("InstallPath") {
        Ok(data) => data,
        Err(_error) => {
            return Err(SteamPathDetectionError::CantReadInRegistry(
                "La section du registre de Steam n'indique pas le dossier d'installation."
                    .to_string(),
            ))
        }
    };

    Ok(steam_path)
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

#[derive(Debug)]
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
fn get_steam_installed_apps(state: tauri::State<'_, LauncherState>) -> Result<Vec<String>, String> {
    let mut core = state.0.lock().unwrap();

    if core.melatonin_info.is_none() {
        match core.update_melatonin_info() {
            Ok(_) => (),
            Err(error) => {
                return Err(format!("{:?}", error));
            }
        }
    }

    if let Some(melatonin_info) = &core.melatonin_info {
        let available_patches = melatonin_info.get_available_patches();
        // Get the Steam path from the registry
        let steam_path = match get_steam_path() {
            Ok(data) => data,
            Err(error) => {
                return Err(format!(
                    "Erreur de récupération du dossier de Steam: {:?}",
                    error
                ))
            }
        };

        let file: File = match File::open(
            Path::new(&steam_path)
                .join("steamapps")
                .join("libraryfolders.vdf"),
        ) {
            Ok(data) => data,
            Err(error) => {
                return Err(format!(
                    "Erreur de lecture du fichier LibraryFolder: {}",
                    error
                ))
            }
        };

        let folders: LibraryFolders = keyvalues_serde::from_reader(file).unwrap();

        let mut games: Vec<String> = Vec::new();

        for folder in folders.folders.values() {
            for app in folder.apps.keys() {
                if available_patches.contains(app) {
                    games.push(app.to_owned());
                }
            }
        }

        Ok(games)
    } else {
        Err("Impossible de récupérer les informations de patch sur le serveur.".to_string())
    }
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
