
@tool
extends GutTest

var world:GEWorldNode

func before_all():
	world = GEWorldNode.new()

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
	#assert_eq(comp.d, [2, "thirty"])
	#assert_eq(comp.e, {"some": "value"})

#endregion

#region Components

class Foo extends GEComponent:
	var a:bool = true:
		get: return getc(&"a")
		set(v): setc(&"a", v)
	var b:int = 35:
		get: return getc(&"b")
		set(v): setc(&"b", v)
	var c:float = 2.36:
		get: return getc(&"c")
		set(v): setc(&"c", v)
	#var d:Array = [2, "thirty"]:
		#get: return getc(&"d")
		#set(v): setc(&"d", v)
	#var e:Dictionary = {"some": "value"}:
		#get: return getc(&"e")
		#set(v): setc(&"e", v)
		
#endregion
