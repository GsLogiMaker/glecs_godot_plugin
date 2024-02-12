
@tool
extends GutTest

var world:GEWorldNode

func before_all():
	world = GEWorldNode.new()
	add_child(world)

func after_all():
	world.free()

#region Tests

func test_prefab():
	world.add_system(
		[Foo, Bar],
		func(_delta:float, foo:Foo, bar:Bar):
			foo.a = true
			foo.b += 1
			foo.c += 1.3
			bar.a.x += foo.c
			bar.a.y += foo.c * 2
			bar.b = PI
			,
	)
	var entity:= world.new_entity_with_prefab("Test", PrefabPck)
	
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	world.run_pipeline(&"process", 0.0)
	
	var foo:Foo = entity.get_component(Foo)
	var bar:Bar = entity.get_component(Bar)
	assert_eq(foo.a, true)
	assert_eq(foo.b, 3)
	assert_almost_eq(foo.c, 3.9, 0.01)
	assert_almost_eq(bar.a, Vector2(7.8, 15.6), Vector2(0.01, 0.01))
	assert_almost_eq(bar.b, PI, 0.01)

#endregion

#region Components

class Foo extends GEComponent:
	const PROPS:= {
		a = TYPE_BOOL,
		b = TYPE_INT,
		c = TYPE_FLOAT,
	}

class Bar extends GEComponent:
	const PROPS:= {
		a = TYPE_VECTOR2,
		b = TYPE_FLOAT,
	}
	
class PrefabPck extends _BaseGEPrefab:
	const COMPONENTS:= [
		Foo,
		Bar,
	]
		
#endregion
