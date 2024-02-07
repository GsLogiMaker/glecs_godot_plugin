
extends GutTest

var world:GEWorldNode = null

func before_all():
	world = GEWorldNode.new()
	add_child(world, true)

func after_all():
	world.queue_free()

func test_add_entity():
	var _entity:= world.new_entity()
	
	# Can't assert, but should be fine as long as it doesn't crash
	assert_null(null)
	
func test_simple_system():
	world.add_system(
		func(foo:Foo):
			foo.setc(&"vec", 2.67)
			,
		[Foo],
	)
	var entity:= world.new_entity([Foo])
	
	world._world_process(1.0)
	
	assert_almost_eq(entity.get_component(Foo).getc(&"vec"), 2.67, 0.01)

class Foo extends GEComponent:
	var vec:= 0.0
