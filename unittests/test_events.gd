
extends GutTest

var world:GlecsWorldNode
var i:= 0

func before_all():
	world = GlecsWorldNode.new()
	add_child(world)

func after_all():
	world.free()

#region Tests

func test_on_add_event():
	i = 0
	
	world.new_event_listener(Glecs.ON_ADD) \
		.with(Ints) \
		.for_each(func(_ints: Ints):
			self.i += 1
			)
	
	var e:= world.new_entity("WithInts", [Ints])
	var e2:= world.new_entity("WithoutInts", [])
	var e3:= world.new_entity("WithInts", [])
	var e4:= world.new_entity("WithoutInts", [])

	e3.add_component(Ints)

	assert_eq(i, 2)

	e.free()
	e2.free()
	e3.free()
	e4.free()

func test_on_add_event_with_objects():
	i = 0
	world.new_event_listener(Glecs.ON_ADD) \
		.with(Textures) \
		.for_each(func(_ints: Textures):
			self.i += 1
			)
	
	var e:= world.new_entity("WithInts", [Textures])
	assert_eq(i, 1)
	assert_eq(e.get_component(Textures).a, null)
	assert_eq(e.get_component(Textures).b, null)

	e.free()
	
	# In this test, the loaded textures will be auto freed by Godot if Glecs
	# does not properly take ownership of them.
	i = 0
	var e2:= world.new_entity("WithTextures")
	e2.add_component(Textures, [load("res://icon.png"), load("res://icon.svg")])
	assert_eq(i, 1)
	assert_eq(e2.get_component(Textures).a, load("res://icon.png"))
	assert_eq(e2.get_component(Textures).b, load("res://icon.svg"))

	e2.free()

#endregion

#region Components

class Ints extends GlecsComponent:
	static func _get_members() -> Dictionary: return {
		a = 0,
		b = 0,
	}
	var a:int:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:int = 25:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Textures extends GlecsComponent:
	static func _get_members() -> Dictionary: return {
		a = null,
		b = null,
	}
	var a:Texture2D:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:Texture2D:
		get: return getc(&"b")
		set(v): setc(&"b", v)

#endregion
