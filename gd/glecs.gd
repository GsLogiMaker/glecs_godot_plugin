
class_name Glecs

## An event that's emitted when component is added to an entity. This is
## before the entity gets set.
static var ON_ADD:= _GlecsBindings._flecs_on_add()
## An event that's emitted when component is removed from an entity.
## TODO: Find out when exactly this is emitted and explain it here.
static var ON_REMOVE:= _GlecsBindings._flecs_on_remove()
## An event that's emitted after any property in a component is changed.
static var ON_SET:= _GlecsBindings._flecs_on_set()
static var UN_SET:= _GlecsBindings._flecs_un_set()
static var MONITOR:= _GlecsBindings._flecs_monitor()
static var ON_DELETE:= _GlecsBindings._flecs_on_delete()
static var ON_TABLE_CREATE:= _GlecsBindings._flecs_on_table_create()
static var ON_TABLE_DELETE:= _GlecsBindings._flecs_on_table_delete()
static var ON_TABLE_EMPTY:= _GlecsBindings._flecs_on_table_empty()
static var ON_TABLE_FILL:= _GlecsBindings._flecs_on_table_fill()
## A tag which designates an entity as a prefab.
static var PREFAB:= _GlecsBindings._flecs_prefab()
## A tag for use in pairs to designate inheritance.
static var IS_A:= _GlecsBindings._flecs_is_a()

## An pipeline that's ran during [method Node._process].
static var PROCESS:= 0
## An pipeline that's ran during [method Node._physics_process].
static var PHYSICS_PROCESS:= 0

class Component extends _GlecsBaseComponent:

	static func _get_members_example_1() -> Array[Dictionary]:
		return [
			{ rid = RID() },
			{ texture = null },
		]
	
	static func _get_members_example_2() -> Dictionary:
		return {
			rid = RID(),
			texture = null,
		}

	static func _registered(world: Glecs.World) -> void:
		pass

	func copy_from_component(from_component: Glecs.Component) -> void:
		_copy_from_component(from_component)
		
	func get_type_name() -> StringName:
		return _get_type_name()
		
	func getc(property: StringName) -> Variant:
		return _getc(property)
		
	func setc(property: StringName, value: Variant) -> void:
		return _setc(property, value)
		
	func delete() -> void:
		_delete()

	func is_valid() -> bool:
		return _is_valid()


## A reference to an entity.
##
## TODO: Explain conversions from Variant to Glecs.Entity
class Entity extends _GlecsBaseEntity:
	
	## Called when the script is registered in [param world]. [br] [br]
	##
	## The main use for this function is for creating prefabs from classes,
	## like so:
	## [codeblock]
	## class MyPrefab extends Glecs.Entity:
	##     static func _registered(world: Glecs.World) -> void:
	##         var my_prefab = Glecs.Entity.from(MyPrefab, world) \
	##             .add_entity(Glecs.PREFAB) \
	##             .add_component(SomeComponent)
	## [/codeblock]
	## For more on prefabs, see [method add_entity]. [br]
	## See also: [method from], [method add_entity], [method add_component]
	static func _registered(world: Glecs.World) -> void:
		pass

	## Creates a new entity in [param world]. [br] [br]
	##
	## Example:
	## [codeblock]
	## var entity = Glecs.Entity.spawn()
	## [/codeblock]
	## See also: [method from]
	static func spawn(world: Glecs.World = null) -> Glecs.Entity:
		return _GlecsBaseEntity._spawn(world)
	
	## Returns a reference to an existing [param entity],
	## in [param world]. [br] [br]
	##
	## Example:
	## [codeblock]
	## var id = Glecs.PREFAB
	## var entity = Glecs.Entity.from(id)
	## [/codeblock]
	## See also: [method spawn], [method Glecs.World.id_from_variant]
	static func from(entity: Variant, world: Glecs.World = null) -> Entity:
		return _GlecsBaseEntity._from(entity, world)

	## Adds [param component] data to this entity, with an optional
	## [param default_value]. [br] [br]
	##
	## Example:
	## [codeblock]
	## var entity = Glecs.Entity.spawn()
	## entity.add_component(MyComponent)
	## [/codeblock]
	## For more on components, see [Glecs.Component]. [br]
	## See also: [method get_component], [method remove_component],
	## [method Glecs.World.id_from_variant]
	func add_component(component:Variant, default_value:Variant=null) -> Entity:
		_add_component(component, default_value)
		return self

	## Returns a reference to this entity's [param component] data. [br] [br]
	## 
	## For more information on components, see: [Glecs.Component]. [br]
	## See also: [method add_component], [method remove_component]
	## , [method Glecs.World.id_from_variant]
	func get_component(component:Variant) -> Component:
		return _get_component(component)

	## Removes [param component] data from this entity. [br] [br]
	## 
	## For more information on components, see [Glecs.Component]. [br]
	## See also: [method add_component], [method has_component],
	## [method Glecs.World.id_from_variant]
	func remove_component(component: Variant) -> Entity:
		_remove_component(component)
		return self

	## Deletes this entity from its ECS world. [br] [br]
	## 
	## This method does not delete this [Glecs.Entity], which is
	## a [RefCounted] object. The lifetime of an entity is completely separate
	## from the lifetime of a [Glecs.Entity], which acts like a reference to an
	## entity.
	## [br] [br]
	## See also: [method is_valid].
	func delete() -> void:
		_delete()

	## Adds a tag or relationship to this entity. [br] [br]
	##
	## Example:
	## [codeblock]
	## var my_tag = Glecs.Entity.spawn()
	## var entity = Glecs.Entity.spawn()
	## 
	## entity.add_entity(my_tag)
	## [/codeblock]
	## See also: [method has_entity], [method remove_entity],
	## [method add_relation], [method Glecs.World.id_from_variant]
	func add_entity(tag: Variant) -> Entity:
		_add_entity(tag)
		return self

	## Returns true if this entity has the given tag or relationship. [br] [br]
	##
	## Example:
	## [codeblock]
	## var my_tag = Glecs.Entity.spawn()
	## var entity = Glecs.Entity.spawn()
	## entity.add_entity(my_tag)
	##
	## assert(entity.has_entity(my_tag) == true)
	## [/codeblock]
	## See also: [method add_entity], [method remove_entity],
	## [method has_relation], [method Glecs.World.id_from_variant]
	func has_entity(tag: Variant) -> bool:
		return _has_entity(tag)

	## Removes a tag or relationship with this entity. [br] [br]
	##
	## Example:
	## [codeblock]
	## var my_tag = Glecs.Entity.spawn()
	## var entity = Glecs.Entity.spawn()
	## entity.add_entity(my_tag)
	##
	## entity.remove_entity(my_tag)
	## assert(entity.has_entity(my_tag) == false)
	## [/codeblock]
	## See also: [method add_entity], [method has_entity],
	## [method remove_relation], [method Glecs.World.id_from_variant]
	func remove_entity(tag: Variant) -> Entity:
		_remove_entity(tag)
		return self

	## Returns the ID of this entity from its world. [br] [br]
	##
	## Example:
	## [codeblock]
	## var first = Glecs.Entity.spawn()
	## var second = Glecs.Entity.from(first)
	##
	## assert(first.get_id() == second.get_id())
	## [/codeblock]
	## See also: [method get_world]
	func get_id() -> int:
		return _get_id()

	## Returns the name of this entity. [br] [br]
	##
	## Example:
	## [codeblock]
	## var entity = Glecs.Entity.spawn()
	## entity.set_name("MyEntity")
	##
	## assert(entity.get_name() == "MyEntity")
	## [/codeblock]
	## See also: [method set_name]
	func get_name() -> String:
		return _get_name()

	## Sets the name of this entity to [param value]. [br] [br]
	##
	## Example:
	## [codeblock]
	## var entity = Glecs.Entity.spawn()
	## entity.set_name("MyEntity")
	##
	## assert(entity.get_name() == "MyEntity")
	## [/codeblock]
	## See also: [method get_name]
	func set_name(value: String) -> Entity:
		_set_name(value)
		return self

	## Adds an entity pair to this entity. [br] [br]
	##
	## Example:
	## [codeblock]
	## var eats = Glecs.Entity.spawn() # Define relation
	## var apples = Glecs.Entity.spawn() # Define relation target
	## var entity = Glecs.Entity.spawn()
	##
	## entity.add_relation(eats, apples)
	## assert(entity.has_relation(eats, apples) == true)
	## [/codeblock]
	## See also: [method has_relation], [method remove_relation],
	## [method add_entity], [method Glecs.World.id_from_variant]
	func add_relation(relation: Variant, with_entity: Variant) -> Entity:
		_add_relation(relation, with_entity)
		return self
	
	## Returns true if this entity has the given entity pair. [br] [br]
	##
	## Example:
	## [codeblock]
	## var eats = Glecs.Entity.spawn() # Define relation
	## var apples = Glecs.Entity.spawn() # Define relation target
	## var entity = Glecs.Entity.spawn()
	## entity.add_relation(eats, apples)
	##
	## assert(entity.has_relation(eats, apples) == true)
	## [/codeblock]
	## See also: [method add_relation], [method remove_relation],
	## [method has_entity], [method Glecs.World.id_from_variant]
	func has_relation(relation: Variant, with_entity: Variant) -> bool:
		breakpoint # TODO: implement Glecs.Entity.has_relation
		return false

	## Removes an entity pair from this entity. [br] [br]
	##
	## Example:
	## [codeblock]
	## var eats = Glecs.Entity.spawn() # Define relation
	## var apples = Glecs.Entity.spawn() # Define relation target
	## var entity = Glecs.Entity.spawn()
	## entity.add_relation(eats, apples)
	##
	## entity.remove_relation(eats, apples)
	## assert(entity.has_relation(eats, apples) == true)
	## [/codeblock]
	## See also: [method add_relation], [method has_relation],
	## [method remove_entity], [method Glecs.World.id_from_variant]
	func remove_relation(relation: Variant, with_entity: Variant) -> Entity:
		_remove_relation(relation, with_entity)
		return self

	## Returns true if this [Glecs.Entity] is a valid reference to an
	## entity. [br] [br]
	##
	## A [Glecs.Entity] is valid if the following are true: [br]
	## - The world is not [code]null[/code]. [br]
	## - The world is not deleted. [br]
	## - The ID of the [Glecs.Entity] is a real entity in the world. [br]
	## - The entity is not deleted.
	##
	## [br] [br]
	## Example:
	## [codeblock]
	## var entity = Glecs.Entity.spawn()
	## entity.delete()
	## assert(is_instance_valid(entity) == true)
	## assert(entity.is_valid() == false)
	## [/codeblock]
	## See also: [method delete]
	func is_valid() -> bool:
		return _is_valid()

	## Returns the world object this entity is in. [br] [br]
	## See also: [method get_id]
	func get_world() -> Glecs.World:
		return _get_world()


class World extends _GlecsBaseWorld:
	
	func _init() -> void:
		Glecs.PROCESS = id_from_variant("Glecs/process")
		Glecs.PHYSICS_PROCESS = id_from_variant("Glecs/physics_process")

	func new_event_listener(
		event:Variant,
	) -> GlecsSystemBuilder:
		return _new_event_listener(event)

	## Converts a [Variant] to an entity ID. [br] [br]
	##
	## How Variants are converts: [br]
	## - [int]: No change. [br]
	## - [float]: Converted to int. [br]
	## - [Vector2i]: Interpreted as a pair. [br]
	## - [Vector2]: Converted to integers, then interpreted as a pair. [br]
	## - [String]: Finds entity by its name. [br]
	## - [StringName]: Finds entity by its name. [br]
	## - [Glecs.Entity]: Calls [method Glecs.Entity.get_id]. [br]
	## - [Glecs.Component]: Throws exception for being too ambiguous. (Should
	## 		it return the component type ID, or the ID of the
	## 		entity its attached to?) [br]
	## - [Script] extending [Glecs.Entity]: Returns the ID registered 
	## 		with the world. [br]
	## - [Script] extending [Glecs.Component]: Returns the ID registered 
	## 		with the world. [br]
	## - Everything else: Throws an exception.
	func id_from_variant(entity: Variant) -> int:
		return _id_from_variant(entity)

	func new_pipeline(
		identifier:Variant,
		additional_parameters:Array[Callable]=[],
	) -> void:
		_new_pipeline(identifier, additional_parameters)

	func new_system(pipeline: Variant = Glecs.PROCESS) -> GlecsSystemBuilder:
		return _new_system(pipeline)

	func new_entity(name:String, with_components:Array[Variant]=[]) -> Glecs.Entity:
		return _new_entity(name, with_components)


class WorldNode extends _GlecsBaseWorldNode:

	func _process(delta: float) -> void:
		run_pipeline(Glecs.PROCESS, delta)

	func _physics_process(delta: float) -> void:
		run_pipeline(Glecs.PHYSICS_PROCESS, delta)

	func new_event_listener(
		event:Variant,
	) -> GlecsSystemBuilder:
		return _new_event_listener(event)

	func id_from_variant(entity: Variant) -> int:
		return _id_from_variant(entity)

	func new_pipeline(
		name: String,
		additional_parameters:Array[Callable]=[],
	) -> Entity:
		return _new_pipeline(name, additional_parameters)

	func new_system(pipeline: Variant = Glecs.PROCESS) -> GlecsSystemBuilder:
		return _new_system(pipeline)

	func new_entity(name:String, with_components:Array[Variant]=[]) -> Glecs.Entity:
		return _new_entity(name, with_components)
