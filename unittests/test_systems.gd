
extends GutTest

var world:GEWorldNode

func before_all():
	world = GEWorldNode.new()
	add_child(world)
	world.new_pipeline(&"test_pipeline")

func after_all():
	world.free()

#region Tests

func test_stuff():
	world.new_system(&"test_pipeline") \
		.with(Bools) \
		.for_each(func(bools:Bools):
			bools.b = bools.a
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

class Bools extends GEComponent:
	const PROPS:= {
		a = TYPE_BOOL,
		b = TYPE_BOOL,
	}
	var a:bool:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:bool:
		get: return getc(&"b")
		set(v): setc(&"b", v)

#endregion
