
extends GutTest

var world:GEWorldNode

func before_all():
	world = GEWorldNode.new()
	add_child(world)
	
	world.add_system(
		[Bools],
		func(_delta:float, x:Bools):
			x.b = x.a
			x.a = not x.b
			,
	)
	world.add_system(
		[Ints],
		func(_delta:float, x:Ints):
			x.b *= 2
			x.a += x.b
			,
	)
	world.add_system(
		[Floats],
		func(_delta:float, x:Floats):
			x.b *= 2.0
			x.a += x.b
			,
	)
	world.add_system(
		[ByteArrays],
		func(_delta:float, x:ByteArrays):
			for i in range(x.a.size()):
				x.a[i] += x.b[i]
			,
	)

func after_all():
	world.free()

#region Tests

func test_pipelines():
	world.new_pipeline(&"1")
	world.new_pipeline(&"2")
	
	var entity:= world.new_entity("Test", [Bools, Ints])
	var ints:Ints = entity.get_component(Ints)
	
	world.add_system(
		[Ints],
		func(ints:Ints):
			ints.a = 25
			,
		&"1",
	)
	world.add_system(
		[Ints],
		func(ints:Ints):
			ints.b = 50
			,
		&"2",
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

func test_bools():
	var entity:= world.new_entity("Test", [Bools])
	
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	
	assert_eq(entity.get_component(Bools).a, true)
	assert_eq(entity.get_component(Bools).b, false)
	
	entity.free()

func test_ints():
	var entity:= world.new_entity("Test", [Ints])
	entity.get_component(Ints).b = 1
	
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	
	assert_eq(entity.get_component(Ints).a, 14)

func test_floats():
	var entity:= world.new_entity("Test", [Floats])
	entity.get_component(Floats).b = 1.2
	
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	
	assert_almost_eq(entity.get_component(Floats).a, 16.8, 0.05)

func test_strings():
	world.add_system(
		[Strings],
		func(_delta:float, x:Strings):
			x.b += "em"
			x.a += x.b
			,
	)
	
	var entity:= world.new_entity("Test")
	entity.add_component(Strings, ["", "po"])
	var strings:Strings = entity.get_component(Strings)
	
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	
	assert_eq(strings.a, "poempoemempoememem")
	assert_eq(strings.b, "poememem")

func test_byte_arrays():
	var entity:= world.new_entity("Test", [ByteArrays])
	entity.get_component(ByteArrays).a = PackedByteArray([1, 2, 3])
	entity.get_component(ByteArrays).b = PackedByteArray([2, 4, 3])
	
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	
	assert_eq(entity.get_component(ByteArrays).a, PackedByteArray([7, 14, 12]))

func test_textures():
	world.add_system(
		[Textures],
		func(_delta:float, x:Textures):
			x.a = x.b
			,
	)
	
	var entity:= world.new_entity("Test", [Textures])
	entity.get_component(Textures).a = null
	entity.get_component(Textures).b = load("res://icon.svg")
	
	# Assert that setting Object to null works
	assert_eq(entity.get_component(Textures).b, load("res://icon.svg"))
	entity.get_component(Textures).b = null
	assert_eq(entity.get_component(Textures).b, null)
	entity.get_component(Textures).b = load("res://icon.svg")
	
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	
	assert_eq(entity.get_component(Textures).a, load("res://icon.svg"))
	assert_eq(entity.get_component(Textures).b, load("res://icon.svg"))

func test_ref_counts():
	var rc:= RefCounted.new()
	assert_eq(rc.get_reference_count(), 1)
	
	var entity:= world.new_entity("Test", [RefCounts])
	
	entity.get_component(RefCounts).a = rc
	assert_eq(rc.get_reference_count(), 2)
	
	entity.get_component(RefCounts).a = null
	assert_eq(rc.get_reference_count(), 1)

func test_arrays():
	world.add_system(
		[Arrays],
		func(_delta:float, x:Arrays):
			for i in mini(x.a.size(), x.b.size()):
				x.b[i] += x.a[i]
			,
	)
	
	var entity:= world.new_entity("Test", [Arrays])
	entity.get_component(Arrays).a = [23, 4, 6]
	entity.get_component(Arrays).b = [1, 2, 1]
	
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	
	assert_eq(entity.get_component(Arrays).a, [23, 4, 6])
	assert_eq(entity.get_component(Arrays).b, [70, 14, 19])


func test_dicts():
	world.add_system(
		[Dicts],
		func(_delta:float, x:Dicts):
			x.b["value"] += x.a["add_by"]
			,
	)
	
	var entity:= world.new_entity("Test", [Dicts])
	entity.get_component(Dicts).a = {"add_by": 5}
	entity.get_component(Dicts).b = {"value": 2}
	
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	
	assert_eq(entity.get_component(Dicts).a, {"add_by":5})
	assert_eq(entity.get_component(Dicts).b, {"value":17})

#endregion

#region Components

class Bools extends GEComponent:
	const PROPS:= {
		a = TYPE_BOOL,
		b = TYPE_BOOL,
	}
	var a:bool:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:bool:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Ints extends GEComponent:
	const PROPS:= {
		a = TYPE_INT,
		b = TYPE_INT,
	}
	var a:int:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:int = 25:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Floats extends GEComponent:
	const PROPS:= {
		a = TYPE_FLOAT,
		b = TYPE_FLOAT,
	}
	var a:float:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:float:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Strings extends GEComponent:
	const PROPS:= {
		a = TYPE_STRING,
		b = TYPE_STRING,
	}
	var a:String:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:String:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class ByteArrays extends GEComponent:
	const PROPS:= {
		a = TYPE_PACKED_BYTE_ARRAY,
		b = TYPE_PACKED_BYTE_ARRAY,
	}
	var a:PackedByteArray:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:PackedByteArray:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Textures extends GEComponent:
	const PROPS:= {
		a = TYPE_OBJECT,
		b = TYPE_OBJECT,
	}
	var a:Texture2D:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:Texture2D:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class RefCounts extends GEComponent:
	const PROPS:= {
		a = TYPE_OBJECT,
		b = TYPE_OBJECT,
	}
	var a:RefCounted:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:RefCounted:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Arrays extends GEComponent:
	const PROPS:= {
		a = TYPE_ARRAY,
		b = TYPE_ARRAY,
	}
	var a:Array:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:Array:
		get: return getc(&"b")
		set(v): setc(&"b", v)

class Dicts extends GEComponent:
	const PROPS:= {
		a = TYPE_DICTIONARY,
		b = TYPE_DICTIONARY,
	}
	var a:Dictionary:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:Dictionary:
		get: return getc(&"b")
		set(v): setc(&"b", v)

#endregion
