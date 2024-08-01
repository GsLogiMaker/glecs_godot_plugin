
extends Control

var mutex:= Mutex.new()

var world:= GlWorld.new()

func _init() -> void:
	pass

func _ready() -> void:
	world.start_rest_api()

func _physics_process(delta: float) -> void:
	world.progress(delta)

func _on_run_tests_pressed() -> void:
	get_tree().change_scene_to_file("res://addons/gut/gui/GutRunner.tscn")


func _on_play_asteroids_pressed() -> void:
	get_tree().change_scene_to_file("res://examples/asteroids/asteroids.tscn")


func _on_compose_release_pressed() -> void:
	pass


func _on_button_pressed() -> void:
	var e:= GlEntity.from(257, world)
	var c:= e.get_component(485)
	if not c:
		prints("Component is null")
		return
	prints("C", c.get_source_id(), c.get_id())
	prints("C member", c.get_member("min"), c.get_member("max"))
	c.set_member("min", c.get_member("min") + 0.1)
	c.set_member("max", c.get_member("max") + 1.1)
	prints("C member post ", c.get_member("min"), c.get_member("max"))
