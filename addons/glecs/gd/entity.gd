
## A reference to an entity.
##
## TODO: Explain conversions from Variant to GlecsEntity
class_name GlecsEntity extends _GlecsBaseEntity

## Called when the script is registered in [param world]. [br] [br]
##
## The main use for this function is for creating prefabs from classes,
## like so:
## [codeblock]
## class MyPrefab extends GlecsEntity:
##     static func _registered(world: GlecsWorldObject) -> void:
##         var my_prefab = GlecsEntity.from(MyPrefab, world) \
##             .add_entity(Glecs.PREFAB) \
##             .add_component(SomeComponent)
## [/codeblock]
## For more on prefabs, see [method add_entity]. [br]
## See also: [method from], [method add_entity], [method add_component]
static func _registered(world: GlecsWorldObject) -> void:
	pass

## Creates a new entity in [param world]. [br] [br]
##
## Example:
## [codeblock]
## var entity = GlecsEntity.spawn()
## [/codeblock]
## See also: [method from]
static func spawn(world: GlecsWorldObject = null) -> GlecsEntity:
	return _GlecsBaseEntity._spawn(world)

## Returns a reference to an existing [param entity],
## in [param world]. [br] [br]
##
## Example:
## [codeblock]
## var id = Glecs.PREFAB
## var entity = GlecsEntity.from(id)
## [/codeblock]
## See also: [method spawn], [method GlecsWorldObject.id_from_variant]
static func from(entity: Variant, world: GlecsWorldObject = null) -> GlecsEntity:
	return _GlecsBaseEntity._from(entity, world)

## Adds [param component] data to this entity, with an optional
## [param default_value]. [br] [br]
##
## Example:
## [codeblock]
## var entity = GlecsEntity.spawn()
## entity.add_component(MyComponent)
## [/codeblock]
## For more on components, see [GlecsComponent]. [br]
## See also: [method get_component], [method remove_component],
## [method GlecsWorldObject.id_from_variant]
func add_component(component:Variant, default_value:Variant=null) -> GlecsEntity:
	_add_component(component, default_value)
	return self

## Returns a reference to this entity's [param component] data. [br] [br]
## 
## For more information on components, see: [GlecsComponent]. [br]
## See also: [method add_component], [method remove_component]
## , [method GlecsWorldObject.id_from_variant]
func get_component(component:Variant) -> GlecsComponent:
	return _get_component(component)

## Removes [param component] data from this entity. [br] [br]
## 
## For more information on components, see [GlecsComponent]. [br]
## See also: [method add_component], [method has_component],
## [method GlecsWorldObject.id_from_variant]
func remove_component(component: Variant) -> GlecsEntity:
	_remove_component(component)
	return self

## Deletes this entity from its ECS world. [br] [br]
## 
## This method does not delete this [GlecsEntity], which is
## a [RefCounted] object. The lifetime of an entity is completely separate
## from the lifetime of a [GlecsEntity], which acts like a reference to an
## entity.
## [br] [br]
## See also: [method is_valid].
func delete() -> void:
	_delete()

## Adds a tag or relationship to this entity. [br] [br]
##
## Example:
## [codeblock]
## var my_tag = GlecsEntity.spawn()
## var entity = GlecsEntity.spawn()
## 
## entity.add_entity(my_tag)
## [/codeblock]
## See also: [method has_entity], [method remove_entity],
## [method add_relation], [method GlecsWorldObject.id_from_variant]
func add_entity(tag: Variant) -> GlecsEntity:
	_add_entity(tag)
	return self

## Returns true if this entity has the given tag or relationship. [br] [br]
##
## Example:
## [codeblock]
## var my_tag = GlecsEntity.spawn()
## var entity = GlecsEntity.spawn()
## entity.add_entity(my_tag)
##
## assert(entity.has_entity(my_tag) == true)
## [/codeblock]
## See also: [method add_entity], [method remove_entity],
## [method has_relation], [method GlecsWorldObject.id_from_variant]
func has_entity(tag: Variant) -> bool:
	return _has_entity(tag)

## Removes a tag or relationship with this entity. [br] [br]
##
## Example:
## [codeblock]
## var my_tag = GlecsEntity.spawn()
## var entity = GlecsEntity.spawn()
## entity.add_entity(my_tag)
##
## entity.remove_entity(my_tag)
## assert(entity.has_entity(my_tag) == false)
## [/codeblock]
## See also: [method add_entity], [method has_entity],
## [method remove_relation], [method GlecsWorldObject.id_from_variant]
func remove_entity(tag: Variant) -> GlecsEntity:
	_remove_entity(tag)
	return self

## Returns the ID of this entity from its world. [br] [br]
##
## Example:
## [codeblock]
## var first = GlecsEntity.spawn()
## var second = GlecsEntity.from(first)
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
## var entity = GlecsEntity.spawn()
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
## var entity = GlecsEntity.spawn()
## entity.set_name("MyEntity")
##
## assert(entity.get_name() == "MyEntity")
## [/codeblock]
## See also: [method get_name]
func set_name(value: String) -> GlecsEntity:
	_set_name(value)
	return self

## Adds an entity pair to this entity. [br] [br]
##
## Example:
## [codeblock]
## var eats = GlecsEntity.spawn() # Define relation
## var apples = GlecsEntity.spawn() # Define relation target
## var entity = GlecsEntity.spawn()
##
## entity.add_relation(eats, apples)
## assert(entity.has_relation(eats, apples) == true)
## [/codeblock]
## See also: [method has_relation], [method remove_relation],
## [method add_entity], [method GlecsWorldObject.id_from_variant]
func add_relation(relation: Variant, with_entity: Variant) -> GlecsEntity:
	_add_relation(relation, with_entity)
	return self

## Returns true if this entity has the given entity pair. [br] [br]
##
## Example:
## [codeblock]
## var eats = GlecsEntity.spawn() # Define relation
## var apples = GlecsEntity.spawn() # Define relation target
## var entity = GlecsEntity.spawn()
## entity.add_relation(eats, apples)
##
## assert(entity.has_relation(eats, apples) == true)
## [/codeblock]
## See also: [method add_relation], [method remove_relation],
## [method has_entity], [method GlecsWorldObject.id_from_variant]
func has_relation(relation: Variant, with_entity: Variant) -> bool:
	breakpoint # TODO: implement GlecsEntity.has_relation
	return false

## Removes an entity pair from this entity. [br] [br]
##
## Example:
## [codeblock]
## var eats = GlecsEntity.spawn() # Define relation
## var apples = GlecsEntity.spawn() # Define relation target
## var entity = GlecsEntity.spawn()
## entity.add_relation(eats, apples)
##
## entity.remove_relation(eats, apples)
## assert(entity.has_relation(eats, apples) == true)
## [/codeblock]
## See also: [method add_relation], [method has_relation],
## [method remove_entity], [method GlecsWorldObject.id_from_variant]
func remove_relation(relation: Variant, with_entity: Variant) -> GlecsEntity:
	_remove_relation(relation, with_entity)
	return self

## Returns true if this [GlecsEntity] is a valid reference to an
## entity. [br] [br]
##
## A [GlecsEntity] is valid if the following are true: [br]
## - The world is not [code]null[/code]. [br]
## - The world is not deleted. [br]
## - The ID of the [GlecsEntity] is a real entity in the world. [br]
## - The entity is not deleted.
##
## [br] [br]
## Example:
## [codeblock]
## var entity = GlecsEntity.spawn()
## entity.delete()
## assert(is_instance_valid(entity) == true)
## assert(entity.is_valid() == false)
## [/codeblock]
## See also: [method delete]
func is_valid() -> bool:
	return _is_valid()

## Returns the world object this entity is in. [br] [br]
## See also: [method get_id]
func get_world() -> GlecsWorldObject:
	return _get_world()
