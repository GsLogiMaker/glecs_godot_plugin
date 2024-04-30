
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
	var foo:= e.get_component(Foo)
	var e2:= w.new_entity("Test", [Foo])
	var foo2:= e2.get_component(Foo)
	
	foo.setc(&"vec", 24.3)
	foo2.setc(&"vec", 125.1)
	
	e2.free()
	assert_eq(e2.is_valid(), false)
	assert_eq(foo2.is_valid(), false)
	
	foo.free()
	assert_eq(e.is_valid(), true)
	assert_eq(foo.is_valid(), false)
	
	w.free()
	assert_eq(is_instance_valid(w), false)
	assert_eq(e.is_valid(), false)
	assert_eq(foo.is_valid(), false)
	

func test_registeration():
	var w:= GlecsWorldNode.new()
	add_child(w)
	
	var e:= w.new_entity("Test", [RegisterationA, RegisterationB])
	
	e.get_component(RegisterationA).set_value(3)
	e.get_component(RegisterationB).set_value(11)
	
	# A system defined in RegistrationA's _registered function should run
	# on GlecsWorldNode's process pipeline
	await get_tree().process_frame # Skip this frame (We are already past the trigger for GlecsWorldNode's process pipeline)
	await get_tree().process_frame # Pipeline process runs first time this frame
	
	assert_almost_eq(e.get_component(RegisterationA).get_result(), 14.0, .001)
	assert_almost_eq(e.get_component(RegisterationB).get_result(), 33.0, .001)

	w.queue_free()


func test_simple_system():
	var a = world.new_system()
	var b = a.with(Foo)
	b.for_each(func(_delta:float, foo:Foo):
		foo.set_value(Vector2(2, 5))
		)
			
	var entity:= world.new_entity("Test", [Foo])
	
	await get_tree().process_frame # Skip this frame
	await get_tree().process_frame # Process is called first time here
	
	assert_eq(entity.get_component(Foo).get_value(), Vector2(2, 5))


func test_default_values():
	var w:= GlecsWorldNode.new()
	var e:= w.new_entity("Test", [WithDefaults])
	assert_eq(e.get_component(WithDefaults).get_int(), WithDefaults._VAR_int)
	assert_eq(e.get_component(WithDefaults).get_string(), WithDefaults._VAR_string)
	assert_eq(e.get_component(WithDefaults).get_script_2(), WithDefaults._VAR_script)

	w.queue_free()


class Foo extends GlecsComponent:
	const _VAR_value:= Vector2.ZERO
	
	func get_value() -> Vector2:
		return getc(&"value")
	
	func set_value(v:Vector2) -> void:
		setc(&"value", v)


class WithDefaults extends GlecsComponent:
	const _VAR_int:= 25
	const _VAR_string:= "Hello world!"
	const _VAR_script:= WithDefaults
	
	func get_int() -> int:
		return getc(&"int")
	func get_string() -> String:
		return getc(&"string")
	func get_script_2() -> Script:
		return getc(&"script")


class RegisterationA extends GlecsComponent:
	const _VAR_value:= 0.0
	const _VAR_result:= 0.0
	
	func get_value() -> float:
		return getc(&"value")
	func set_value(v:float) -> void:
		setc(&"value", v)
	func get_result() -> float:
		return getc(&"result")
	func set_result(v:float) -> void:
		setc(&"result", v)
	
	static func _registered(world:GlecsWorld):
		world.new_system() \
			.with(RegisterationA) \
			.with(RegisterationB) \
			.for_each(func(_delta:float, reg_a:RegisterationA, reg_b:RegisterationB):
				reg_a.set_result(reg_a.get_value() + reg_b.get_value())
				)


class RegisterationB extends GlecsComponent:
	const _VAR_value:= 0.0
	const _VAR_result:= 0.0
	
	func get_value() -> float:
		return getc(&"value")
	func set_value(v:float) -> void:
		setc(&"value", v)
	func get_result() -> float:
		return getc(&"result")
	func set_result(v:float) -> void:
		setc(&"result", v)
	
	static func _registered(world:GlecsWorld):
		world.new_system() \
			.with(RegisterationA) \
			.with(RegisterationB) \
			.for_each(func(_delta:float, reg_a:RegisterationA, reg_b:RegisterationB):
				reg_b.set_result(reg_a.get_value() * reg_b.get_value())
				)


class NoDefine extends GlecsComponent:
	pass


