extends Control

@export var steam_registry_path: String


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	#print("Steam installation path:")
	#print(await windows_get_steam_path())
	var steam_utils = SteamUtils.new(steam_registry_path)
	for i in await steam_utils.get_games():
		var app_card = preload("res://app_card.tscn").instantiate()
		app_card.get_node("name").text = i.name
		app_card.get_node("image").texture = i.card_image
		
		$MarginContainer/CardsContainer.add_child(app_card)
	
# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass
