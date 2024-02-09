
extends GutTest

var world:GEWorldNode

func before_all():
	world = GEWorldNode.new()

func after_all():
	world.free()

#region Tests

func test_bools():
	world.add_system(
		[],
		func():
			,
	)
	
	var man:= world.new_entity("Man", [])
	var apple:= world.new_entity("Apple", [])
	man.add_relation("eats", apple)
	var cow:= world.new_entity("Cow", [])
	var grass:= world.new_entity("Grass", [])
	cow.add_relation("eats", grass)
	
	world.run_process(&"process", 0.0)
	
	assert_eq(true, true)
	
	man.free()
	apple.free()
#endregion

#region Components

class Bools extends GEComponent:
	const PROPS:= {
		a = TYPE_BOOL,
		b = TYPE_BOOL,
	}

#endregion
