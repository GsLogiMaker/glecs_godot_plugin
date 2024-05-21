
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

func test_new_entity_with_unregistered_component():
	var e:GlecsEntity = world.new_entity("Test", [Unregistered])
	assert_eq(e.get_component(Unregistered).value, 0)

func test_creating_entity_by_new():
	# Test that an entity is invalidated by being deleted
	var e:= GlecsEntity.spawn(world.as_object())
	assert_eq(e.is_valid(), true)
	e.delete()
	assert_eq(e.is_valid(), false)
	
	# Test that an entity is invalidated by its world being deleted
	var w:= GlecsWorldObject.new()
	var e2:= GlecsEntity.spawn(w)
	assert_eq(e2.is_valid(), true)
	w.free()
	assert_eq(e2.is_valid(), false)

func test_entity_from():
	var id:= 0
	if true:
		var tmp_entity = GlecsEntity.spawn(world.as_object())
		tmp_entity.set_name(&"Cool Name")
		id = tmp_entity.get_id()
	assert_ne(id, 0)
	
	var e:= GlecsEntity.from(id, world.as_object())
	assert_ne(e, null)
	assert_eq(e.get_name(), &"Cool Name")

func test_entity_created_in_singleton():
	var e:= GlecsEntity.spawn()
	assert_eq(e.is_valid(), true)
	
	var e2:= GlecsEntity.spawn(GlecsSingleton)
	assert_eq(e2.is_valid(), true)
	
	assert_eq(e.get_world(), e2.get_world())
	
	e.delete()
	e2.delete()

#endregion

#region Classes

class Foo extends GlecsComponent:
	static func _get_members() -> Dictionary: return {
		value = 0.0,
	}
	var value:float:
		get: return getc(&"value")
		set(v): setc(&"value", v)

class Stringy extends GlecsComponent:
	static func _get_members() -> Dictionary: return {
		a = "",
		b = "",
	}
	var a:String:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:String:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Unadded extends GlecsComponent:
	static func _get_members() -> Dictionary: return {
		value = 0,
	}
	var value:int:
		get: return getc(&"value")
		set(v): setc(&"value", v)

class Unregistered extends GlecsComponent:
	static func _get_members() -> Dictionary: return {
		value = 0,
	}
	var value:int:
		get: return getc(&"value")
		set(v): setc(&"value", v)

#endregion
