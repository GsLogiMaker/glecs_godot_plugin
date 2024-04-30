
extends GutTest

var world:Glecs.WorldNode
var i:= 0

func before_all():
	world = Glecs.WorldNode.new()
	add_child(world)

func after_all():
	world.free()

#region Tests

func test_on_add_event():
	i = 0
	
	world.new_event_listener(world.ON_ADD_EVENT) \
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
	world.new_event_listener(world.ON_ADD_EVENT) \
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

class Ints extends Glecs.Component:
	const _VAR_a:= 0
	const _VAR_b:= 0
	var a:int:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:int = 25:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Textures extends Glecs.Component:
	const _VAR_a:= null
	const _VAR_b:= null
	var a:Texture2D:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:Texture2D:
		get: return getc(&"b")
		set(v): setc(&"b", v)
	
	static func _registered(world:Glecs.World) -> void:
		world.new_event_listener(world.ON_ADD_EVENT) \
			.with(Textures) \
			.for_each(func(t: Textures):
				prints("Added Textures", t.a, t.b)
				)
		
		world.new_event_listener(world.ON_SET_EVENT) \
			.with(Textures) \
			.for_each(func(t: Textures):
				prints("Set Textures", t.a, t.b)
				)

#endregion
