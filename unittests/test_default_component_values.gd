
@tool
extends GutTest

var world:GEWorldNode

func before_all():
	world = GEWorldNode.new()

func after_all():
	world.free()

#region Tests

func test_default_values():
	var entity:= world.new_entity([Foo])
	
	world._world_process(0.0)
	world._world_process(0.0)
	world._world_process(0.0)
	
	var comp:Foo = entity.get_component(Foo)
	
	assert_eq(comp.get_a(), false)
	assert_eq(comp.get_b(), 0)
	assert_almost_eq(comp.get_c(), 0.0, 0.01)

#endregion

#region Components

class Foo extends GEComponent:
	const PROPS:= {
		a = TYPE_BOOL,
		b = TYPE_INT,
		c = TYPE_FLOAT,
	}
		
	func get_a() -> bool:
		return getc(&"a")
	func set_a(v:bool) -> void:
		setc(&"a", v)
		
	func get_b() -> int:
		return getc(&"b")
	func set_b(v:int) -> void:
		setc(&"b", v)
		
	func get_c() -> float:
		return getc(&"c")
	func set_c(v:float) -> void:
		setc(&"c", v)
		
#endregion
