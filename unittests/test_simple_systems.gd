
extends GutTest

var world:GEWorldNode

func before_all():
	world = GEWorldNode.new()
	
	world.add_system(
		func(x:Bools):
			x.b = x.a
			x.a = not x.b
			,
		[Bools],
	)
	world.add_system(
		func(x:Ints):
			prints("INTS")
			x.b *= 2
			x.a += x.b
			,
		[Ints],
	)
	world.add_system(
		func(x:Floats):
			x.b *= 2.0
			x.a += x.b
			,
		[Floats],
	)
	world.add_system(
		func(x:ByteArrays):
			for i in range(x.a.size()):
				x.a[i] += x.b[i]
			,
		[ByteArrays],
	)

func after_all():
	world.free()

#region Tests

func test_bools():
	var entity:= world.new_entity([Bools])
	
	world._world_process(0.0)
	world._world_process(0.0)
	world._world_process(0.0)
	
	assert_eq(entity.get_component(Bools).a, true)
	assert_eq(entity.get_component(Bools).b, false)
	
	entity.free()

func test_ints():
	var entity:= world.new_entity([Ints])
	entity.get_component(Ints).b = 1
	
	world._world_process(0.0)
	world._world_process(0.0)
	world._world_process(0.0)
	
	assert_eq(entity.get_component(Ints).a, 14)

func test_floats():
	var entity:= world.new_entity([Floats])
	entity.get_component(Floats).b = 1.2
	
	world._world_process(0.0)
	world._world_process(0.0)
	world._world_process(0.0)
	
	assert_almost_eq(entity.get_component(Floats).a, 16.8, 0.05)

func test_strings():
	world.add_system(
		func(x:Strings):
			x.b += "em"
			x.a += x.b
			,
		[Strings],
	)
	
	var entity:= world.new_entity([Strings])
	var strings:Strings = entity.get_component(Strings)
	strings.a = ""
	strings.b = "po"
	
	world._world_process(0.0)
	world._world_process(0.0)
	world._world_process(0.0)
	
	assert_eq(strings.a, "poempoemempoememem")
	assert_eq(strings.b, "poememem")

func test_byte_arrays():
	var entity:= world.new_entity([ByteArrays])
	entity.get_component(ByteArrays).a = PackedByteArray([1, 2, 3])
	entity.get_component(ByteArrays).b = PackedByteArray([2, 4, 3])
	
	world._world_process(0.0)
	world._world_process(0.0)
	world._world_process(0.0)
	
	assert_eq(entity.get_component(ByteArrays).a, PackedByteArray([7, 14, 12]))

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

#endregion
