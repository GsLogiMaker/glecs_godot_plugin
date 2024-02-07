
extends GutTest

var world:GEWorldNode = null

func before_all():
	world = GEWorldNode.new()
	add_child(world, true)

func after_all():
	world.free()

#region Tests

func test_component_get_and_set():
	var e:_BaseGEEntity = world.new_entity([Foo])
	
	var foo:Foo = e.get_component(Foo)
	assert_almost_eq(foo.value, 0.0, 0.01)
	
	foo.value = 2.3
	assert_almost_eq(foo.value, 2.3, 0.01)
	
func test_get_unadded_component():
	var e:_BaseGEEntity = world.new_entity()
	assert_eq(e.get_component(Unadded), null)

func test_new_entity_with_unregistered_component():
	var e:_BaseGEEntity = world.new_entity([Unregistered])
	assert_eq(e.get_component(Unregistered).value, 0)

#endregion

#region Classes

class Foo extends GEComponent:
	var value:float:
		get: return getc(&"value")
		set(v): setc(&"value", v)

class Unadded extends GEComponent:
	var value:int:
		get: return getc(&"value")
		set(v): setc(&"value", v)

class Unregistered extends GEComponent:
	var value:int:
		get: return getc(&"value")
		set(v): setc(&"value", v)

#endregion
