@tool
extends EditorPlugin


func _enter_tree() -> void:
	add_autoload_singleton("Glecs", (GlecsWorld as Script).resource_path)


func _exit_tree() -> void:
	remove_autoload_singleton("Glecs")
