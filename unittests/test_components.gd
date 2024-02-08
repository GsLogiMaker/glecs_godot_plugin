
extends GutTest

var world:GEWorldNode = null

func before_all():
	world = GEWorldNode.new()
	add_child(world, true)

func after_all():
	world.free()

func test_add_entity():
	var _entity:= world.new_entity("Test")
	
	# Can't assert, but should be fine as long as it doesn't crash
	assert_null(null)
	
func test_world_deletion():
	var w:= GEWorldNode.new()
	var e:= w.new_entity("Test", [Foo])
	var foo = e.get_component(Foo)
	var e2:= w.new_entity("Test", [Foo])
	var foo2 = e2.get_component(Foo)
	
	foo.setc(&"vec", 24.3)
	foo2.setc(&"vec", 125.1)
	
	e2.free()
	assert_eq(is_instance_valid(e2), false)
	assert_eq(is_instance_valid(foo2), false)
	
	foo.free()
	assert_eq(is_instance_valid(foo), true)
	
	w.free()
	assert_eq(is_instance_valid(w), false)
	assert_eq(is_instance_valid(e), false)
	assert_eq(is_instance_valid(foo), false)
	
func test_simple_system():
	world.add_system(
		func(foo:Foo):
			foo.set_value(Vector2(2, 5))
			,
		[Foo],
	)
	var entity:= world.new_entity("Test", [Foo])
	
	world._world_process(1.0)
	
	assert_eq(entity.get_component(Foo).get_value(), Vector2(2, 5))

func test_error_no_define():
	var entity:= world.new_entity("Test", [NoDefine])
	assert_ne(entity, null)

func test_error_wrong_type_define():
	var entity:= world.new_entity("Test", [WrongTypeDefine])
	assert_ne(entity, null)

class Foo extends GEComponent:
	const PROPS:= {
		value = TYPE_VECTOR2,
	}
	
	func get_value() -> Vector2:
		return getc(&"value")
	
	func set_value(v:Vector2) -> void:
		setc(&"value", v)

class NoDefine extends GEComponent:
	pass

class WrongTypeDefine extends GEComponent:
	const PROPS:= ""
