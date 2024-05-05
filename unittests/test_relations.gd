
extends GutTest

var world:GlecsWorldNode

func before_all():
	world = GlecsWorldNode.new()
	add_child(world)

func after_all():
	world.free()

#region Tests

func test_bools():
	# TODO: Query for relations
	world.new_system().for_each(func(_delta): pass)
	var apple:= world.new_entity("Apple", [])
	world.new_entity("Eats", [])
	
	var man:= world.new_entity("Man", [])
	man.add_relation("Eats", apple)
	
	var cow:= world.new_entity("Cow", [])
	var grass:= world.new_entity("Grass", [])
	cow.add_relation("Eats", grass)
	
	world.run_pipeline(Glecs.PROCESS, 0.0)
	
	assert_eq(true, true)
	
	man.free()
	apple.free()
#endregion

#region Components

class Bools extends GlecsComponent:
	static func _get_members() -> Dictionary: return {
		a = false,
		b = false,
	}

#endregion
