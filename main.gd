extends Control


func _on_run_tests_pressed() -> void:
	pass # Replace with function body.


func _on_play_asteroids_pressed() -> void:
	get_tree().change_scene_to_file("res://example/asteroids/asteroids.tscn")
