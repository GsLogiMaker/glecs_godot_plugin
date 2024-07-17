
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
	var apple:= GlecsEntity.spawn(world.as_object()) \
		.set_name("Apple")
		
	GlecsEntity.spawn(world.as_object()).set_name("Eats")
	
	var man:= GlecsEntity.spawn(world.as_object()) \
		.set_name("Man") \
		.add_relation("Eats", apple)
	
	var cow:= GlecsEntity.spawn(world.as_object()) \
		.set_name("Cow")
	var grass:= GlecsEntity.spawn(world.as_object()) \
		.set_name("Grass")
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
