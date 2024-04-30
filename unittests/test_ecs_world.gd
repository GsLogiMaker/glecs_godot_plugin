
extends GutTest

var world:GlecsWorldNode = null

func before_all():
	world = GlecsWorldNode.new()
	add_child(world, true)

func after_all():
	world.free()

func test_add_entity():
	var _entity:= world.new_entity("Test")
	
	# Can't assert, but should be fine as long as it doesn't crash
	assert_null(null)
	
func test_world_deletion():
	var w:= GlecsWorldNode.new()
	
	var e:= w.new_entity("Test", [Foo])
	var foo:Foo = e.get_component(Foo)
	
	var e2:= w.new_entity("Test", [Foo])
	var foo2:Foo = e2.get_component(Foo)
	
	foo.setc(&"vec", 24.3)
	foo2.setc(&"vec", 125.1)
	
	assert_eq(e.is_valid(), true)
	assert_eq(foo.is_valid(), true)
	assert_eq(e2.is_valid(), true)
	assert_eq(foo2.is_valid(), true)
	
	e2.free()
	assert_eq(e.is_valid(), true)
	assert_eq(foo.is_valid(), true)
	assert_eq(e2.is_valid(), false)
	assert_eq(foo2.is_valid(), false)
	
	foo.free()
	assert_eq(e.is_valid(), true)
	assert_eq(foo.is_valid(), false)
	assert_eq(e2.is_valid(), false)
	assert_eq(foo2.is_valid(), false)
	
	w.free()
	assert_eq(is_instance_valid(w), false)
	assert_eq(e.is_valid(), false)
	assert_eq(foo.is_valid(), false)
	
func test_simple_system():
	world.new_system() \
		.with(Foo) \
		.for_each(func(_delta:float, foo:Foo):
			foo.setc(&"vec", 2.67)
			)
			
	var entity:= world.new_entity("Test", [Foo])
	
	world.run_pipeline(world.PROCESS_PIPELINE, 1.0)
	
	assert_almost_eq(entity.get_component(Foo).getc(&"vec"), 2.67, 0.01)

class Foo extends GlecsComponent:
	const _VAR_vec:= 0.0
