
@tool
extends GutTest

var world:Glecs.WorldNode

func before_all():
	world = Glecs.WorldNode.new()
	add_child(world)

func after_all():
	world.free()

#region Tests

func test_prefab():
	world.new_system() \
		.with(Foo) \
		.with(Bar) \
		.for_each(func(_delta:float, foo:Foo, bar:Bar):
			foo.b += 1
			foo.c += 1.3
			bar.a.x += foo.c
			bar.a.y += foo.c * 2
			bar.b = PI
			)
			
	var entity:= Glecs.Entity.spawn(world.as_object())
	entity.add_entity(world.pair(world.IS_A_TAG, MyPrefab))
	
	# Test inhereted componets exist entity
	var foo:Foo = entity.get_component(Foo)
	var bar:Bar = entity.get_component(Bar)
	assert_ne(foo, null)
	assert_ne(bar, null)

	# Test default values of inhereted  components
	assert_eq(foo.a, true)
	assert_eq(foo.b, 23)
	assert_almost_eq(foo.c, 2.33, 0.001)
	assert_almost_eq(bar.a, Vector2(2, 1.1), Vector2(0.001, 0.001))
	assert_almost_eq(bar.b, 5.6, 0.001)

	# Test process with inhereted components
	world.run_pipeline(world.PROCESS_PIPELINE, 0.0)
	assert_eq(foo.b, 24)
	assert_almost_eq(foo.c, 3.63, 0.001)
	assert_almost_eq(bar.a, Vector2(2+foo.c, 1.1+(foo.c*2)), Vector2(0.001, 0.001))
	assert_almost_eq(bar.b, PI, 0.001)

#endregion

#region Components

class Foo extends Glecs.Component:
	const _VAR_a:= false
	const _VAR_b:= 0
	const _VAR_c:= 0.0

class Bar extends Glecs.Component:
	const _VAR_a:= Vector2.ZERO
	const _VAR_b:= 0.0
	
class MyPrefab extends Glecs.Entity:
	
	static func _registered(world:Glecs.World) -> void:
		var p:= Glecs.Entity.from(MyPrefab, world)
		p.add_entity(world.PREFAB_TAG)
		p.add_component(Foo, [true, 23, 2.33])
		p.add_component(Bar, [Vector2(2, 1.1), 5.6])
		
#endregion
