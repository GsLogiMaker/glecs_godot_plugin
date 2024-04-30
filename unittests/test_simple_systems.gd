
extends GutTest

var world:GlecsWorldNode

func before_all():
	world = GlecsWorldNode.new()
	add_child(world)

func after_all():
	world.free()

#region Tests

func test_pipelines():
	world.new_pipeline(&"1")
	world.new_pipeline(&"2")
	
	var entity:= world.new_entity("Test", [Bools, Ints])
	var ints:Ints = entity.get_component(Ints)
	
	world.new_system(&"1") \
		.with(Ints) \
		.for_each(func(x:Ints):
			ints.a = 25
			)
	world.new_system(&"2") \
		.with(Ints) \
		.for_each(func(x:Ints):
			ints.b = 50
			)
	
	ints.a = 0
	ints.b = 0
	assert_eq(entity.get_component(Ints).a, 0)
	assert_eq(entity.get_component(Ints).b, 0)
	world.run_pipeline(&"1", 0.0)
	assert_eq(entity.get_component(Ints).a, 25)
	assert_eq(entity.get_component(Ints).b, 0)
	
	ints.a = 0
	ints.b = 0
	assert_eq(entity.get_component(Ints).a, 0)
	assert_eq(entity.get_component(Ints).b, 0)
	world.run_pipeline(&"2", 0.0)
	assert_eq(entity.get_component(Ints).a, 0)
	assert_eq(entity.get_component(Ints).b, 50)
	
	entity.free()

func test_bools():
	world.new_system() \
		.with(Bools) \
		.for_each(func(_delta, x:Bools):
			x.b = x.a
			x.a = not x.b
			)
	
	var entity:= world.new_entity("Test", [Bools])
	
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	
	assert_eq(entity.get_component(Bools).a, true)
	assert_eq(entity.get_component(Bools).b, false)
	
	entity.free()

func test_ints():
	world.new_system() \
		.with(Ints) \
		.for_each(func(_delta, x:Ints):
			x.b *= 2
			x.a += x.b
			)
	
	var entity:= world.new_entity("Test", [Ints])
	entity.get_component(Ints).b = 1
	
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	
	assert_eq(entity.get_component(Ints).a, 14)
	
	entity.free()

func test_floats():
	world.new_system() \
		.with(Floats) \
		.for_each(func(_delta, x:Floats):
			x.b *= 2
			x.a += x.b
			)
	
	var entity:= world.new_entity("Test", [Floats])
	entity.get_component(Floats).b = 1.2
	
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	
	assert_almost_eq(entity.get_component(Floats).a, 16.8, 0.05)
	
	entity.free()

func test_strings():
	world.new_system() \
		.with(Strings) \
		.for_each(func(_delta, x:Strings):
			x.b += "em"
			x.a += x.b
			)
	
	var entity:= world.new_entity("Test")
	entity.add_component(Strings, ["", "po"])
	var strings:Strings = entity.get_component(Strings)
	assert_eq(strings.a, "")
	assert_eq(strings.b, "po")
	
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	
	assert_eq(strings.a, "poempoemempoememem")
	assert_eq(strings.b, "poememem")
	
	entity.free()

func test_byte_arrays():
	world.new_system() \
		.with(ByteArrays) \
		.for_each(func(_delta, x:ByteArrays):
			for i in range(x.a.size()):
				x.a[i] += x.b[i]
			)
			
	var entity:= world.new_entity("Test", [ByteArrays])
	entity.get_component(ByteArrays).a = PackedByteArray([1, 2, 3])
	entity.get_component(ByteArrays).b = PackedByteArray([2, 4, 3])
	
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	
	assert_eq(entity.get_component(ByteArrays).a, PackedByteArray([7, 14, 12]))
	
	entity.free()

func test_textures():
	world.new_system() \
		.with(Textures) \
		.for_each(func(_delta, x:Textures):
			x.a = x.b
			)
	
	var entity:= world.new_entity("Test", [Textures])
	entity.get_component(Textures).a = null
	entity.get_component(Textures).b = load("res://icon.svg")
	
	# Assert that setting Object to null works
	assert_eq(entity.get_component(Textures).b, load("res://icon.svg"))
	entity.get_component(Textures).b = null
	assert_eq(entity.get_component(Textures).b, null)
	entity.get_component(Textures).b = load("res://icon.svg")
	
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	
	assert_eq(entity.get_component(Textures).a, load("res://icon.svg"))
	assert_eq(entity.get_component(Textures).b, load("res://icon.svg"))
	
	entity.free()

func test_ref_counts():
	var rc:= RefCounted.new()
	assert_eq(rc.get_reference_count(), 1)
	
	var entity:= world.new_entity("Test", [RefCounts])
	
	entity.get_component(RefCounts).a = rc
	assert_eq(rc.get_reference_count(), 2)
	
	entity.get_component(RefCounts).a = null
	assert_eq(rc.get_reference_count(), 1)
	
	entity.free()

func test_arrays():
	world.new_system() \
		.with(Arrays) \
		.for_each(func(_delta, x:Arrays):
			for i in mini(x.a.size(), x.b.size()):
				x.b[i] += x.a[i]
			)
	
	var entity:= world.new_entity("Test", [Arrays])
	entity.get_component(Arrays).a = [23, 4, 6]
	entity.get_component(Arrays).b = [1, 2, 1]
	
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	
	assert_eq(entity.get_component(Arrays).a, [23, 4, 6])
	assert_eq(entity.get_component(Arrays).b, [70, 14, 19])
	
	entity.free()

func test_dicts():
	world.new_system() \
		.with(Dicts) \
		.for_each(func(_delta, x:Dicts):
			x.b["value"] += x.a["add_by"]
			)
	
	var entity:= world.new_entity("Test", [Dicts])
	entity.get_component(Dicts).a = {"add_by": 5}
	entity.get_component(Dicts).b = {"value": 2}
	
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	
	assert_eq(entity.get_component(Dicts).a, {"add_by":5})
	assert_eq(entity.get_component(Dicts).b, {"value":17})
	
	entity.free()

#endregion

#region Components

class Bools extends GlecsComponent:
	const _VAR_a:= false
	const _VAR_b:= false
	var a:bool:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:bool:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Ints extends GlecsComponent:
	const _VAR_a:= 0
	const _VAR_b:= 0
	var a:int:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:int = 25:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Floats extends GlecsComponent:
	const _VAR_a:= 0.0
	const _VAR_b:= 0.0
	var a:float:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:float:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Strings extends GlecsComponent:
	const _VAR_a:= ""
	const _VAR_b:= ""
	var a:String:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:String:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class ByteArrays extends GlecsComponent:
	const _VAR_a:PackedByteArray = []
	const _VAR_b:PackedByteArray = []
	var a:PackedByteArray:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:PackedByteArray:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Textures extends GlecsComponent:
	const _VAR_a:Texture2D = null
	const _VAR_b:Texture2D = null
	var a:Texture2D:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:Texture2D:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class RefCounts extends GlecsComponent:
	const _VAR_a:RefCounted = null
	const _VAR_b:RefCounted = null
	var a:RefCounted:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:RefCounted:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Arrays extends GlecsComponent:
	const _VAR_a:= []
	const _VAR_b:= []
	var a:Array:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:Array:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Dicts extends GlecsComponent:
	const _VAR_a:= {}
	const _VAR_b:= {}
	var a:Dictionary:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:Dictionary:
		get: return getc(&"b")
		set(v): setc(&"b", v)

#endregion
