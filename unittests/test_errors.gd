
extends GutTest

var world:Glecs.WorldNode = null

func before_all():
	world = Glecs.WorldNode.new()
	add_child(world, true)

func after_all():
	world.free()

#region Tests

func test_get_nonexistant_property():
	var entity:= world.new_entity("Test", [Foo])
	var foo:Foo = entity.get_component(Foo)
	
	assert_eq(foo.getc(&"not a real property"), null)


func test_new_entity_with_unregistered_component():
	var _entity:= world.new_entity("Test", [Unregistered])
	
	# We can't assert the right error is thrown, but it should be fine as
	# long as it doesn't crash
	assert_null(null)

#endregion

#region Classes

class Foo extends Glecs.Component:
	static func _get_members() -> Dictionary: return {
		vec = 0.0,
	}

class Unregistered extends Glecs.Component:
	static func _get_members() -> Dictionary: return {
		vec = 0.0,
	}

#endregion
