class_name SteamUtils

var steam_registry_path: String

func _init(_steam_registry_path):
	steam_registry_path = _steam_registry_path

enum SteamPathDetection {
	INVALID_REGISTRY_PATH = 0,
	INCORRECT_VALUE = 1,
	STEAM_NOT_FOUND = 2,
	CANT_FIND_GAMES = 3,
}

func get_games_path():
	var steam_path = await windows_get_steam_path()
	
	var steamapps_path = steam_path.path_join("steamapps")
		
	if not DirAccess.dir_exists_absolute(steam_path):
		return SteamPathDetection.STEAM_NOT_FOUND
	
	var dir_access = DirAccess.open(steamapps_path)
	if not dir_access.file_exists("libraryfolders.vdf"):
		return SteamPathDetection.CANT_FIND_GAMES
	
	var library_folder_file = FileAccess.open(steamapps_path.path_join("libraryfolders.vdf"), FileAccess.READ)
	var vdf_data = library_folder_file.get_as_text()
	library_folder_file.close()
	
	var parsed = VdfParser.new().parse_vdf(vdf_data)
	
	for i in parsed["libraryfolders"]:
		for appid in parsed["libraryfolders"][i]["apps"]:
			if dir_access.file_exists("appmanifest_" + appid + ".acf"):
				print("appmanifest_" + appid + ".acf")
	
# On Windows, return the Steam installation folder path
func windows_get_steam_path():
	var out = []
	await OS.execute("reg", ["query", steam_registry_path, "/v", "InstallPath"], out)
	
	var output = "".join(out)
	
	if len(output) == 0:
		return SteamPathDetection.STEAM_NOT_FOUND
	
	var info_line = ""
	for i in output.split("\r\n"):
		if "InstallPath" in i:
			info_line = i
			
	if len(info_line) == 0:
		return SteamPathDetection.INVALID_REGISTRY_PATH
	
	var regex_expr = RegEx.new()
	regex_expr.compile("[A-Z]:[\\/\\\\].*")
	
	var path = regex_expr.search(info_line)
	if path:
		return path.get_string()
	
	return SteamPathDetection.INCORRECT_VALUE
