@tool
extends EditorPlugin

const GlecsWorldSingleton:Script = preload("res://addons/glecs/gd/world_node_singleton.gd")

func _enter_tree() -> void:
	add_autoload_singleton("GlecsWorld", (GlecsWorldSingleton as Script).resource_path)


func _exit_tree() -> void:
	remove_autoload_singleton("GlecsWorld")
