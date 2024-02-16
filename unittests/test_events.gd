
extends GutTest

var world:GEWorldNode
var i:= 0

func before_all():
	world = GEWorldNode.new()
	add_child(world)

func after_all():
	world.free()

#region Tests

func test_on_add_event():
	i = 0
	
	world.new_event_listener(&"on_add") \
		.with(Ints) \
		.for_each(func(_ints:Ints):
			self.i += 1
			)
	
	var e:= world.new_entity("WithInts", [Ints])
	var e2:= world.new_entity("WithoutInts", [])
	var e3:= world.new_entity("WithInts", [])
	var e4:= world.new_entity("WithoutInts", [])

	e3.add_component(Ints)

	assert_eq(i, 2)

	e.free()
	e2.free()
	e3.free()
	e4.free()

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
