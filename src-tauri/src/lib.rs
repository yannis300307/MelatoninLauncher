#![allow(unused)]

use directories::ProjectDirs;
use reqwest::blocking;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
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
    steamid: String,
    download_link: String,
    card_image: String,
    traductors: Vec<String>,
    artits: Vec<String>,
    testers: Vec<String>,
}

enum GetPatchesInfoError {
    cantReadFile,
    networkError,
}

impl MelatoninInfo {
    fn get_from_remote() -> Result<MelatoninInfo, GetPatchesInfoError> {
        let patches_info_response = blocking::get(format!("{}infos.json", TM_DOMAINE));

        match patches_info_response {
            Ok(data) => {
                let infos: MelatoninInfo = match data.json() {
                    Ok(data) => data,
                    Err(_error) => return Err(GetPatchesInfoError::cantReadFile),
                };

                Ok(infos)
            }
            Err(_error) => Err(GetPatchesInfoError::networkError),
        }
    }

    fn get_available_patches(&self) -> Vec<String> {
        let mut patches_id: Vec<String> = Vec::new();
        for patch in self.patches.values() {
            patches_id.push(patch.steamid.to_owned());
        }

        patches_id
    }
}

#[tauri::command]
fn get_steam_installed_apps(state: tauri::State<'_, LauncherState>) -> Vec<String> {
    state.0.lock().unwrap().update_melatonin_info();

    // Get the Steam path from the registry
    let steam_path = get_steam_path();

    let library_folder_content = fs::read_to_string(
        Path::new(&steam_path)
            .join("steamapps")
            .join("libraryfolders.vdf"),
    )
    .expect("Can't access library folder file.");

    let library_folder_parsed = vdf_parser::parse_vdf_text(&library_folder_content)
        .expect("Error handling library folder VDF file.");

    let mut games: Vec<String> = Vec::new();

    match library_folder_parsed.value {
        vdf_parser::VdfValue::Block(library_folder_block) => {
            for library in library_folder_block.values() {
                match &library.value {
                    vdf_parser::VdfValue::Block(library_block) => match library_block.get("apps") {
                        Some(app) => match &app.value {
                            vdf_parser::VdfValue::Block(game_id) => {
                                for game in game_id.keys() {
                                    games.push(game.to_string());
                                }
                            }
                            vdf_parser::VdfValue::String(_) => {
                                panic!("Library VDF file is invalid.");
                            }
                        },
                        None => panic!("Library VDF file is invalid."),
                    },
                    vdf_parser::VdfValue::String(_) => {
                        panic!("Library VDF file is invalid.");
                    }
                }
            }
        }
        vdf_parser::VdfValue::String(_) => {
            panic!("Library VDF file is invalid.");
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
