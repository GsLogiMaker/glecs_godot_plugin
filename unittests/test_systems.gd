
extends GutTest

var world:GlecsWorldNode

func before_all():
	world = GlecsWorldNode.new()
	add_child(world)
	world.new_pipeline(&"test_pipeline")

func after_all():
	world.free()

#region Tests

func test_stuff():
	world.new_system(&"test_pipeline") \
		.with(Bools) \
		.for_each(func(boo:Bools):
			boo.b = boo.a
			)
	
	var e:= world.new_entity("Test", [Bools])
	var bools:Bools = e.get_component(Bools)
	bools.a = true
	bools.b = false
	
	assert_eq(bools.a, true)
	assert_eq(bools.b, false)
	
	world.run_pipeline(&"test_pipeline", 1.0)
	
	assert_eq(bools.a, true)
	assert_eq(bools.b, true)
	

#endregion

#region Components

class Bools extends GlecsComponent:
	static func _get_members() -> Dictionary: return {
		a = false,
		b = false,
	}
	var a:bool:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:bool:
		get: return getc(&"b")
		set(v): setc(&"b", v)

#endregion
