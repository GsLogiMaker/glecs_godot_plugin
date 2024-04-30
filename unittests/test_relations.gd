
extends GutTest

var world:Glecs.WorldNode

func before_all():
	world = Glecs.WorldNode.new()
	add_child(world)

func after_all():
	world.free()

#region Tests

func test_bools():
	# TODO: Query for relations
	world.new_system().for_each(func(_delta): pass)
	
	var man:= world.new_entity("Man", [])
	var apple:= world.new_entity("Apple", [])
	man.add_relation("eats", apple)
	var cow:= world.new_entity("Cow", [])
	var grass:= world.new_entity("Grass", [])
	cow.add_relation("eats", grass)
	
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	
	assert_eq(true, true)
	
	man.free()
	apple.free()
#endregion

#region Components

class Bools extends Glecs.Component:
	const _VAR_a:= false
	const _VAR_b:= false

#endregion
