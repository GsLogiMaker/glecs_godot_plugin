
## A reference to an entity.
##
## TODO: Explain conversions from Variant to GlecsEntity

class_name GlecsEntity extends _GlecsEntity

## Called when the script is registered with Glecs.
static func _registered(world:GlecsWorld) -> void:
	pass

## Creates a new entity.
static func spawn(world:GlecsWorld = null) -> GlecsEntity:
	return _GlecsEntity._spawn(world)

## Creates a reference to an existing entity from the
## given [Variant].
static func from(entity:Variant, world:GlecsWorld = null) -> GlecsEntity:
	return _GlecsEntity._from(entity, world)

## Adds component data to this entity, with optional custom default value.
func add_component(component:Variant, default_value:Variant=null) -> void:
	_add_component(component, default_value)

## Returns a reference to a component.
func get_component(component:Variant) -> GlecsComponent:
	return _get_component(component)

## Removes component data from this entity.
func remove_component(component: Variant) -> void:
	_remove_component(component)

## Deletes this entity from the ECS world.
##
## Note: this does not delete the [GlecsEntity] object, which is
## a reference counted reference to the entity.
func delete() -> void:
	_delete()

## Adds an other entity as a tag or relationship to this entity.
func add_entity(tag: Variant) -> void:
	_add_entity(tag)

## Returns true if this entity has the given tag or relationship.
func has_entity(tag: Variant) -> bool:
	return _has_entity(tag)

## Removes the given tag or relationship with this entity.
func remove_entity(tag: Variant) -> void:
	_remove_entity(tag)

## Returns the ID of this entity according to its world.
func get_id() -> int:
	return _get_id()

## Returns the name of this entity.
func get_name() -> String:
	return _get_name()

## Sets the name of this entity.
func set_name(value: String) -> void:
	_set_name(value)

## Adds to this entity a relationship between the two given entites.
func add_relation(relation: Variant, with_entity: Variant) -> void:
	_add_relation(relation, with_entity)

## Removes from this entity a relationship between the two given entites.
func remove_relation(relation: Variant, with_entity: Variant) -> void:
	_remove_relation(relation, with_entity)

## Returns true if this entity reference is valid.
##
## An entity reference is valid if the following are true:
## - The ID of the reference is a real entity in the world.
## - The entity is not deleted.
## - The world is not delete.
func is_valid() -> bool:
	return _is_valid()

## Returns the world object this entity is in.
func get_world() -> GlecsWorld:
	return _get_world()
