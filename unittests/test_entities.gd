
extends GutTest

var world:GlecsWorldNode = null

func before_all():
	world = GlecsWorldNode.new()
	add_child(world, true)

func after_all():
	world.free()

#region Tests

func test_component_get_and_set():
	var e:GlecsEntity = world.new_entity("Test", [Foo])
	
	var foo:Foo = e.get_component(Foo)
	assert_almost_eq(foo.value, 0.0, 0.01)
	
	foo.value = 2.3
	assert_almost_eq(foo.value, 2.3, 0.01)

func test_component_string_get_and_set():
	var e:GlecsEntity = world.new_entity("Test", [Stringy])
	
	var foo:Stringy = e.get_component(Stringy)
	foo.a = "po"
	foo.b = "em"
	assert_eq(foo.a, "po")
	
	foo.a += foo.b
	
	assert_eq(foo.a, "poem")
	assert_eq(foo.b, "em")
	
func test_get_unadded_component():
	var e:GlecsEntity = world.new_entity("Test")
	assert_eq(e.get_component(Unadded), null)

func test_new_entity_with_unregistered_component():
	var e:GlecsEntity = world.new_entity("Test", [Unregistered])
	assert_eq(e.get_component(Unregistered).value, 0)

func test_creating_entity_by_new():
	# Test that an entity is invalidated by being deleted
	var e:= GlecsEntity.spawn(0, world.as_object())
	assert_eq(e.is_valid(), true)
	e.delete()
	assert_eq(e.is_valid(), false)
	
	# Test that an entity is invalidated by its world being deleted
	var w:= GlecsWorld.new()
	var e2:= GlecsEntity.spawn(0, w)
	assert_eq(e2.is_valid(), true)
	w.free()
	assert_eq(e2.is_valid(), false)

#endregion

#region Classes

class Foo extends GlecsComponent:
	const _VAR_value:= 0.0
	var value:float:
		get: return getc(&"value")
		set(v): setc(&"value", v)

class Stringy extends GlecsComponent:
	const _VAR_a:= ""
	const _VAR_b:= ""
	var a:String:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:String:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Unadded extends GlecsComponent:
	const _VAR_value:= 0
	var value:int:
		get: return getc(&"value")
		set(v): setc(&"value", v)

class Unregistered extends GlecsComponent:
	const _VAR_value:= 0
	var value:int:
		get: return getc(&"value")
		set(v): setc(&"value", v)

#endregion
