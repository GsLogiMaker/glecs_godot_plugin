
extends GutTest

var world:GlecsWorldNode

func before_all():
	world = GlecsWorldNode.new()
	add_child(world)
	world.new_pipeline(&"test")

func after_all():
	world.free()

#region Tests

func test_optional_terms():
	var w:= world.as_object()
	w.new_pipeline("1")
	w.new_pipeline("2")
	w.new_pipeline("3")
	
	var empty:= GlecsEntity.spawn(w) \
		.set_name("Empty")
	var just_ints:= GlecsEntity.spawn(w) \
		.set_name("JustInts") \
		.add_component(Ints)
	var just_bools:= GlecsEntity.spawn(w) \
		.set_name("JustBools") \
		.add_component(Bools)
	var all:= GlecsEntity.spawn(w) \
		.set_name("All") \
		.add_component(Ints) \
		.add_component(Bools)
	
	var data:Dictionary = {i=0, ints=0, bools=0}
	var callable:= func(ints, bools):
		data.i += 1
		if ints:
			data.ints += 1
		if bools:
			data.bools += 1
		prints(data)
		
	data.i = 0
	data.ints = 0
	data.bools = 0
	w.new_system(&"1") \
		.maybe_with(Ints) \
		.maybe_with(Bools) \
		.for_each(callable)
	w.run_pipeline(&"1", 0.0)
	assert_eq(data.ints, 2)
	assert_eq(data.bools, 2)
	
	data.i = 0
	data.ints = 0
	data.bools = 0
	w.new_system(&"2") \
		.with(Ints) \
		.maybe_with(Bools) \
		.for_each(callable)
	w.run_pipeline(&"2", 0.0)
	assert_eq(data.i, 2)
	assert_eq(data.ints, 2)
	assert_eq(data.bools, 1)
	
	data.i = 0
	data.ints = 0
	data.bools = 0
	w.new_system(&"3") \
		.maybe_with(Ints) \
		.with(Bools) \
		.for_each(callable)
	w.run_pipeline(&"3", 0.0)
	assert_eq(data.i, 2)
	assert_eq(data.ints, 1)
	assert_eq(data.bools, 2)

#endregion

#region Components

class Bools extends GlecsComponent:
	static func _get_members() -> Dictionary: return {
		a = false,
		b = false,
	}
	var a:bool:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:bool:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Ints extends GlecsComponent:
	static func _get_members() -> Dictionary: return {
		a = 0,
		b = 0,
	}
	var a:int:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:int:
		get: return getc(&"b")
		set(v): setc(&"b", v)

#endregion
