extends Node2D

const COMPONETS:GDScript = preload("res://components.gd")
const DICTIONARY:GDScript = preload("res://components/dictionary.gd")

const TEST_SIMPLE_SYSTEMS:GDScript = preload("res://unittests/test_simple_systems.gd")

const GUT_RUNNER:GDScript = preload("res://addons/gut/gui/GutRunner.gd")


func _ready() -> void:
	var test = TEST_SIMPLE_SYSTEMS.new()
	test.test_th
