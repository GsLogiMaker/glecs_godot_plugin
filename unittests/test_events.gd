
extends GutTest

var world:GEWorldNode
var i:= 0

func before_all():
	world = GEWorldNode.new()
	add_child(world)

func after_all():
	world.free()

#region Tests

func test_on_add_event():
	i = 0
	
	world.new_event_listener(&"on_add") \
		.with(Ints) \
		.for_each(func(_ints:Ints):
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

#endregion

#region Components

class Ints extends GEComponent:
	const _VAR_a:= 0
	const _VAR_b:= 0
	var a:int:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:int = 25:
		get: return getc(&"b")
		set(v): setc(&"b", v)

#endregion
