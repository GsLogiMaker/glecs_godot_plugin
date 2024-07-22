
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

func test_or_operation_terms():
	var w:= world.as_object()
	
	var data:= {ints=0, bools=0}
	w.new_system("test") \
		.or_with(Bools).with(Ints) \
		.for_each(func(bools:Bools, ints:Ints):
			prints(bools, ints)
			#if term is Ints:
				#data.ints += 1
			#if term is Bools:
				#data.bools += 1
			)
	
	GlecsEntity.spawn(w).add_component(Ints)
	GlecsEntity.spawn(w).add_component(Bools)
	GlecsEntity.spawn(w).add_component(Ints).add_component(Bools)
	
	w.run_pipeline("test", 0.0)
	
	prints(data)
	
func test_get_gd_component_data():
	var w:= world.as_object()
	 
	var component_properties_id = _GlecsComponents.id_gd_component_data(w)
	var ints:= GlecsEntity.from(
		_GlecsComponents.define(w, Ints, "Ints"),
		w,
	)
	
	assert_eq(ints.has_entity(component_properties_id), true)
	
	var q:= _GlecsQueries.new_query()
	# Component
	_GlecsQueries.push_term(q, _GlecsBindings.id_component())
	_GlecsQueries.set_term_access_mode(q, 3)
	# With component named "ComponentProperties"
	_GlecsQueries.push_term(q, 0)
	_GlecsQueries.set_term_access_mode(q, 3)
	_GlecsQueries.set_term_first_id(q, _GlecsBindings.id_pred_eq())
	_GlecsQueries.set_term_second_id(q, _GlecsBindings.id_is_name())
	_GlecsQueries.set_term_second_name(q, "ComponentProperties")

	var data:= {count = 0}
	_GlecsQueries.iterate(w, q, func(x=null, y=null):
		data.count += 1
		prints(x, y)
		)
	
	assert_eq(data.count, 1)

func test_query_component_by_script():
	var w:= world.as_object()
	 
	var component_properties_id = _GlecsComponents.id_gd_component_data(w)
	var ints:= GlecsEntity.from(
		_GlecsComponents.define(w, Ints, "Ints"),
		w,
	)
	
	assert_eq(ints.has_entity(component_properties_id), true)
	
	var q:= _GlecsQueries.new_query()
	# Component
	_GlecsQueries.push_term(q, 0)
	#_GlecsQueries.set_term_first_id(q, _GlecsBindings.id)
	_GlecsQueries.set_term_access_mode(q, 3)
	# With component named "ComponentProperties"
	_GlecsQueries.push_term(q, 0)
	_GlecsQueries.set_term_access_mode(q, 3)
	_GlecsQueries.set_term_first_id(q, _GlecsBindings.id_pred_eq())
	_GlecsQueries.set_term_second_id(q, _GlecsBindings.id_is_name())
	_GlecsQueries.set_term_second_name(q, "ComponentProperties")

	var data:= {count = 0}
	_GlecsQueries.iterate(w, q, func(x=null, y=null):
		data.count += 1
		prints(x, y)
		)
	
	assert_eq(data.count, 1)

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
