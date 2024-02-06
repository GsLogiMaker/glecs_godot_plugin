
@tool
extends GutTest

var world:GEWorldNode

func before_all():
	world = GEWorldNode.new()
	
	world.add_component(&"Foo", Foo)

func after_all():
	world.queue_free()

#region Tests

func test_default_values():
	var entity:= world.new_entity([Foo])
	
	world._world_process(0.0)
	world._world_process(0.0)
	world._world_process(0.0)
	
	var comp:Foo = entity.get_component(Foo)
	
	assert_eq(comp.a, true)
	assert_eq(comp.b, 35)
	assert_almost_eq(comp.c, 2.36, 0.01)
	assert_eq(comp.d, [2, "thirty"])
	assert_eq(comp.e, {"some": "value"})

#endregion

#region Components

class Foo extends GEComponent:
	var a:bool = true:
		get: return get(&"a")
		set(v): set(&"a", v)
	var b:int = 35:
		get: return get(&"b")
		set(v): set(&"b", v)
	var c:float = 2.36:
		get: return get(&"c")
		set(v): set(&"c", v)
	var d:Array = [2, "thirty"]:
		get: return get(&"d")
		set(v): set(&"d", v)
	var e:Dictionary = {"some": "value"}:
		get: return get(&"e")
		set(v): set(&"e", v)
		
#endregion
