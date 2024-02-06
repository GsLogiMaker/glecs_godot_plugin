
extends GutTest

var world:GEWorldNode = null

func before_all():
	world = GEWorldNode.new()
	add_child(world, true)

func after_all():
	world.queue_free()

#region Tests

func test_get_nonexistant_property():
	var entity:= world.new_entity([Foo])
	var foo:Foo = entity.get_component(Foo)
	
	assert_eq(foo.getc(&"not a real property"), null)

func test_set_nonexistant_property():
	var entity:= world.new_entity([Foo])
	var foo:Foo = entity.get_component(Foo)
	
	foo.setc(&"not a real property", 1)

func test_set_wrong_type():
	var entity:= world.new_entity([Foo])
	var foo:Foo = entity.get_component(Foo)
	
	foo.setc(&"vec", true)

func test_new_entity_with_unregistered_component():
	prints("Unregistered", Unregistered)
	prints("Unregistered.get_instance_id()", (Unregistered as Script).get_instance_id())
	var entity:= world.new_entity([Unregistered])

#endregion

#region Classes

class Foo extends GEComponent:
	var vec:= 0.0

class Unregistered extends GEComponent:
	var vec:= 0.0

#endregion
