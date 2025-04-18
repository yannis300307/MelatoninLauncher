#![allow(unused)]

use directories::ProjectDirs;
use reqwest::blocking;
use reqwest::header::TE;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use tokio::sync::Mutex;

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

#[derive(Deserialize, Debug, Clone)]
struct MelatoninInfo {
    contacts: HashMap<String, Contact>,
    patches: HashMap<String, Patch>,
}

#[derive(Deserialize, Debug, Clone)]
struct Contact {
    name: String,
    role: String,
    icon: String,
    contacts: Vec<ContactSocial>,
}

#[derive(Deserialize, Debug, Clone)]
struct ContactSocial {
    name: String,
    icon: String,
    url: String,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
struct Patch {
    name: String,
    patch_version: String,
    steam_id: String,
    download_link: String,
    card_image: String,
    traductors: Vec<String>,
    artits: Vec<String>,
    testers: Vec<String>,
    icon: String,
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
struct SteamAppInfo {
    appid: String,
    installdir: String,
}

#[derive(Deserialize, Debug)]
struct Library {
    path: PathBuf,
    apps: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RegisteredApp {
    name: String,
    installation_path: String,
    global_id: String,
    available_patch: Patch,
    patch_activated: bool,
}

impl MelatoninInfo {
    async fn get_from_remote() -> Result<MelatoninInfo, GetPatchesInfoError> {
        let patches_info_response = reqwest::get(format!("{}infos.json", TM_DOMAINE)).await;

        match patches_info_response {
            Ok(data) => {
                let infos: MelatoninInfo = match data.json().await {
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

    fn link_steam_id_to_global(&self) -> HashMap<String, String> {
        let mut patches_link: HashMap<String, String> = HashMap::new();
        for (global_id, patch) in &self.patches {
            patches_link.insert(patch.steam_id.to_owned(), global_id.to_owned());
        }
        patches_link
    }
}

fn get_steam_app_info(steam_id: &String) -> Result<SteamAppInfo, String> {
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

    for folder in folders.folders.values() {
        for current_steam_id in folder.apps.keys() {
            if (*current_steam_id == *steam_id) {
                let file: File = match File::open(
                    Path::new(&folder.path)
                        .join("steamapps")
                        .join(format!("appmanifest_{}.acf", steam_id)),
                ) {
                    Ok(data) => data,
                    Err(error) => {
                        return Err("Erreur de lecture du fichier AppManifest. Si votre jeu est sur un disque externe, assurez vous qu'il soit bien connecté.".to_string())
                    }
                };
                let mut app_info: SteamAppInfo = keyvalues_serde::from_reader(file).unwrap();
                println!("{:?}", &app_info.installdir);
                app_info.installdir = Path::new(&folder.path)
                    .join("steamapps")
                    .join("common")
                    .join(app_info.installdir)
                    .into_os_string()
                    .into_string()
                    .unwrap();
                return Ok(app_info);
            }
        }
    }

    Err("Impossible d'ouvrir AppManifest. Veuillez réinstaller le jeu.".to_string())
}

fn get_installed_apps_steam_id() -> Result<Vec<String>, String> {
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

    let mut apps_id: Vec<String> = Vec::new();

    for folder in folders.folders.values() {
        println!("{:?}", folder);
        apps_id.extend(folder.apps.keys().map(|i| i.to_owned()));
    }

    Ok(apps_id)
}

#[tauri::command]
async fn register_app_from_steam(
    state: tauri::State<'_, LauncherState>,
    global_id: String,
) -> Result<(), String> {
    let mut core = state.0.lock().await;

    if core.melatonin_info.is_none() {
        match core.update_melatonin_info().await {
            Ok(_) => (),
            Err(error) => {
                return Err(format!("{:?}", error));
            }
        }
    };

    if let Some(melatonin_info) = &core.melatonin_info {
        let linked_steam_id = melatonin_info.link_steam_id_to_global();

        if let Some(patch) = melatonin_info.patches.get(&global_id) {
            let name = patch.name.to_owned();
            let available_patch = patch.to_owned();
            let app = get_steam_app_info(&patch.steam_id)?;

            core.registered_apps.insert(
                global_id.clone(),
                RegisteredApp {
                    global_id: global_id.clone(),
                    name,
                    installation_path: app.installdir,
                    available_patch,
                    patch_activated: false,
                },
            );
        }
    };

    core.save_registered_apps()?;

    Ok(())
}

#[tauri::command]
async fn get_remote_available_patches(
    state: tauri::State<'_, LauncherState>,
) -> Result<Vec<serde_json::Value>, String> {
    let mut core = state.0.lock().await;

    if core.melatonin_info.is_none() {
        match core.update_melatonin_info().await {
            Ok(_) => (),
            Err(error) => {
                return Err(format!("{:?}", error));
            }
        }
    };
    println!("hello");

    if let Some(melatonin_info) = &core.melatonin_info {
        let installed_steam_ids = get_installed_apps_steam_id();
        let linked_steam_id = melatonin_info.link_steam_id_to_global();

        if let Some(melatonin_info) = &core.melatonin_info {
            let mut games: Vec<serde_json::Value> = Vec::new();

            for (global_id, patch) in melatonin_info.patches.iter() {
                let app_info = melatonin_info.patches.get(global_id).unwrap();
                let installed_on_steam = {
                    if let Ok(ids) = &installed_steam_ids {
                        ids.contains(&app_info.steam_id)
                    } else {
                        false
                    }
                };

                let registered = core.get_game_is_registered(global_id);
                let patch_activated = core.get_game_patch_activated(global_id);

                games.push(json!({
                    "global_id": global_id,
                    "steam_id": app_info.steam_id,
                    "icon": app_info.icon,
                    "name": app_info.name,
                    "patch_version": app_info.patch_version,
                    "download_link": app_info.download_link,
                    "card_image": app_info.card_image,
                    "traductors": app_info.traductors,
                    "artits": app_info.artits,
                    "testers": app_info.testers,
                    "installed_on_steam": installed_on_steam,
                    "registered": registered,
                    "patch_activated": patch_activated,
                }));
            }
            Ok(games)
        } else {
            Err("Impossible de récupérer les informations de patch sur le serveur.".to_string())
        }
    } else {
        Err("Vous semblez ne pas être connecté à Internet.".to_string())
    }
}

struct MelatoninLauncher {
    melatonin_info: Option<MelatoninInfo>,
    registered_apps: HashMap<String, RegisteredApp>,
}

impl MelatoninLauncher {
    fn new() -> MelatoninLauncher {
        MelatoninLauncher {
            melatonin_info: None,
            registered_apps: HashMap::new(),
        }
    }

    async fn update_melatonin_info(&mut self) -> Result<(), GetPatchesInfoError> {
        let updated = MelatoninInfo::get_from_remote().await;
        match updated {
            Ok(info) => {
                self.melatonin_info = Some(info);

                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    fn get_game_is_registered(&self, global_id: &String) -> bool {
        for app in self.registered_apps.keys() {
            if *app == *global_id {
                return true;
            }
        }
        false
    }

    fn get_game_patch_activated(&self, global_id: &String) -> bool {
        for app in &self.registered_apps {
            if *app.0 == *global_id {
                return app.1.patch_activated;
            }
        }
        false
    }

    fn load_registered_apps(&mut self) -> Result<(), String> {
        if let Some(project_dir) =
            directories::ProjectDirs::from("fr", "TeamMelatonin", "MelatoninLauncher")
        {
            let registered_apps_file_path = project_dir
                .config_dir()
                .join(PathBuf::from("registered_apps.ron"));

            let mut file: File = match File::open(registered_apps_file_path) {
                Ok(data) => data,
                Err(error) => {
                    return Err("Impossible d'ouvrir le fichier des apps enregistrées.".to_string())
                }
            };

            let mut file_content = String::new();
            file.read_to_string(&mut file_content);

            let data_result: Result<HashMap<String, RegisteredApp>, ron::de::SpannedError> =
                ron::from_str(&file_content);

            if let Ok(data) = data_result {
                for patch in data {
                    self.registered_apps.insert(patch.0, patch.1);
                }
            } else {
                return Err("Impossible de charger le fichier des apps installées.".to_string());
            }
        }
        Ok(())
    }

    fn save_registered_apps(&self) -> Result<(), String> {
        println!("saving registered apps...");
        if let Some(project_dir) =
            directories::ProjectDirs::from("fr", "TeamMelatonin", "MelatoninLauncher")
        {
            if let Err(error) = fs::create_dir_all(project_dir.config_dir()) {
                return Err(error.to_string());
            };
            let registered_apps_file_path = project_dir
                .config_dir()
                .join(PathBuf::from("registered_apps.ron"));

            let mut file: File = match File::create(registered_apps_file_path) {
                Ok(data) => data,
                Err(error) => return Err("Impossible d'ouvrir le fichier d'enregistrement des applications enregistrées.".to_string()),
            };

            if let Ok(serialised) = ron::to_string(&self.registered_apps) {
                file.write_all(serialised.as_bytes());
            }
            println!("OK");
        };
        Ok(())
    }
}

fn detect_patch_cached(global_id: &String) -> Option<PathBuf> {
    if let Some(project_dir) =
        directories::ProjectDirs::from("fr", "TeamMelatonin", "MelatoninLauncher")
    {
        let cach_dir = project_dir.cache_dir();

        let file_path = cach_dir.join(format!("{}.zip", global_id));

        if !file_path.exists() {
            return None;
        }
        Some(file_path)
    } else {
        None
    }
}

async fn download_patch(app: &RegisteredApp) -> Result<PathBuf, String> {
    if let Some(project_dir) =
        directories::ProjectDirs::from("fr", "TeamMelatonin", "MelatoninLauncher")
    {
        let cache_dir = project_dir.cache_dir();

        if !Path::exists(cache_dir) {
            fs::create_dir_all(cache_dir);
        }

        let file_path = cache_dir.join(cache_dir.join(format!("{}.zip", app.global_id)));

        if let Ok(data) = reqwest::get(format!(
            "{}{}",
            TM_DOMAINE, app.available_patch.download_link
        ))
        .await
        {
            if let Ok(mut file) = fs::File::create(&file_path) {
                if let Ok(bytes) = data.bytes().await {
                    if file.write_all(&bytes).is_err() {
                        return Err("Impossible d'écrire le fichier de patch.".to_string());
                    }
                    Ok(file_path)
                } else {
                    Err("Impossible de lire les données du patch.".to_string())
                }
            } else {
                Err("Impossible d'écrire le fichier de patch.".to_string())
            }
        } else {
            Err("Impossible de télécharger le patch. Veuillez vérifier votre connection à Internet.".to_string())
        }
    } else {
        Err("Impossible de trouver le dossier de cache.".to_string())
    }
}

fn get_local_original_files(app: &RegisteredApp) -> Option<PathBuf> {
    if let Some(project_dir) =
        directories::ProjectDirs::from("fr", "TeamMelatonin", "MelatoninLauncher")
    {
        let save_dir = project_dir
            .cache_dir()
            .join("original_files")
            .join(&app.global_id);

        if save_dir.is_dir() {
            return Some(save_dir);
        }
    }
    None
}

#[tauri::command]
async fn disable_patch(
    state: tauri::State<'_, LauncherState>,
    global_id: String,
) -> Result<(), String> {
    let mut core = state.0.lock().await;

    if core.melatonin_info.is_none() {
        match core.update_melatonin_info().await {
            Ok(_) => (),
            Err(error) => {
                return Err(format!("{:?}", error));
            }
        }
    };

    if let Some(app) = core.registered_apps.get_mut(&global_id) {
        if let Some(path) = get_local_original_files(app) {
            for entry in walkdir::WalkDir::new(&path).into_iter().flatten() {
                if path == entry.path() {
                    continue;
                }
                if Path::is_file(entry.path()) {
                    let parent_dir = entry.path().parent().expect("Your file system is weird.");
                    if let Ok(relative_path) = parent_dir.strip_prefix(&path) {
                        let destination = Path::new(&app.installation_path).join(relative_path);

                        if fs::copy(entry.path(), destination.join(entry.file_name())).is_err() {
                            return Err("Impossible de copier les fichiers originaux.".to_string());
                        }
                    } else {
                        return Err("Les données du jeu semblent corrompues.".to_string())
                    }
                }
            }
        }
        app.patch_activated = false;
        core.save_registered_apps();
        Ok(())
    } else {
        Err("Le jeu semble ne pas exister.".to_string())
    }
}

#[tauri::command]
async fn enable_patch(
    state: tauri::State<'_, LauncherState>,
    global_id: String,
) -> Result<(), String> {
    let mut core = state.0.lock().await;

    if core.melatonin_info.is_none() {
        match core.update_melatonin_info().await {
            Ok(_) => (),
            Err(error) => {
                return Err(format!("{:?}", error));
            }
        }
    };

    if let Some(app) = core.registered_apps.get_mut(&global_id) {
        let path: PathBuf;
        if let Some(path_) = detect_patch_cached(&global_id) {
            println!("Patch already cached.");
            path = path_;
        } else {
            println!("Downloading patch...");
            path = download_patch(app).await?;
            println!("Patch downloaded!");
        }

        if let Ok(content) = fs::read(path) {
            // Save original files
            if let Some(project_dir) =
                directories::ProjectDirs::from("fr", "TeamMelatonin", "MelatoninLauncher")
            {
                let save_dir = project_dir
                    .cache_dir()
                    .join("original_files")
                    .join(&app.global_id);
                fs::create_dir_all(&save_dir);
                if let Ok(mut zip) = zip::read::ZipArchive::new(Cursor::new(&content)) {
                    for i in 0..zip.len() {
                        let mut file = zip.by_index(i).unwrap();
                        println!("Filename: {}", file.name());
                        let file_path = Path::new(&app.installation_path).join(file.name());
                        let dest_path = save_dir.join(file.name());
                        fs::create_dir_all(dest_path.parent().unwrap());
                        fs::copy(file_path, dest_path); // TODO: extract here
                    }
                };
            }

            if let Err(error) = zip_extract::extract(
                Cursor::new(&content),
                Path::new(&app.installation_path),
                true,
            ) {
                return Err(format!("Impossible de décompresser le patch : {:?}", error));
            }

            app.patch_activated = true;
            core.save_registered_apps()?;

            println!("Installed!");
        } else {
            return Err("Impossible de lire le patch. Il est-peut être corompu.".to_string());
        }

        Ok(())
    } else {
        Err("Le jeu semble ne pas exister.".to_string())
    }
}

#[tauri::command]
async fn loading_finished(state: tauri::State<'_, LauncherState>) -> Result<(), String> {
    let mut core = state.0.lock().await;

    core.load_registered_apps()?;

    println!("Loading finished");

    Ok(())
}

struct LauncherState(pub Mutex<MelatoninLauncher>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(LauncherState(Mutex::new(MelatoninLauncher::new())))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_remote_available_patches,
            loading_finished,
            register_app_from_steam,
            enable_patch,
            disable_patch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
