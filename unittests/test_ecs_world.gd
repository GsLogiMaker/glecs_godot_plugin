
extends GutTest

var world:GEWorldNode = null

func before_all():
	world = GEWorldNode.new()
	add_child(world, true)

func after_all():
	world.queue_free()

func test_add_entity():
	var entity:= world.new_entity()
	
func test_simple_system():
	world.add_system(
		func(foo:Foo):
			#foo.vec += 1.0
			prints("system", foo)
			,
		[Foo],
	)
	var entity:= world.new_entity([Foo])
	
	world._world_process(1.0)

class Foo extends GEComponent:
	var vec:= 0.0
