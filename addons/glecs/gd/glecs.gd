
class_name Glecs

static var ON_ADD:= _GlecsBindings._flecs_on_add()
static var ON_REMOVE:= _GlecsBindings._flecs_on_remove()
static var ON_SET:= _GlecsBindings._flecs_on_set()
static var UN_SET:= _GlecsBindings._flecs_un_set()
static var MONITOR:= _GlecsBindings._flecs_monitor()
static var ON_DELETE:= _GlecsBindings._flecs_on_delete()
static var ON_TABLE_CREATE:= _GlecsBindings._flecs_on_table_create()
static var ON_TABLE_DELETE:= _GlecsBindings._flecs_on_table_delete()
static var ON_TABLE_EMPTY:= _GlecsBindings._flecs_on_table_empty()
static var ON_TABLE_FILL:= _GlecsBindings._flecs_on_table_fill()
static var PREFAB:= _GlecsBindings._flecs_prefab()
static var IS_A:= _GlecsBindings._flecs_is_a()

class Component extends _GlecsBaseComponent:

	static func _registered(world:Glecs.World) -> void:
		pass

	func copy_from_component(from_component: Glecs.Component) -> void:
		_copy_from_component(from_component)
		
	func get_type_name() -> StringName:
		return _get_type_name()
		
	func getc(property: StringName) -> Variant:
		return _getc(property)
		
	func setc(property: StringName, value:Variant) -> void:
		return _setc(property, value)
		
	func delete() -> void:
		_delete()

	func is_valid() -> bool:
		return _is_valid()


## A reference to an entity.
##
## TODO: Explain conversions from Variant to Glecs.Entity
class Entity extends _GlecsBaseEntity:
	
	## Called when the script is registered with Glecs.
	static func _registered(world:Glecs.World) -> void:
		pass

	## Creates a new entity.
	static func spawn(world:Glecs.World = null) -> Glecs.Entity:
		return _GlecsBaseEntity._spawn(world)
	
	## Creates a reference to an existing entity from the
	## given [Variant].
	static func from(entity:Variant, world:Glecs.World = null) -> Glecs.Entity:
		return _GlecsBaseEntity._from(entity, world)

	func add_component(component:Variant, default_value:Variant=null) -> Entity:
		_add_component(component, default_value)
		return self

	## Returns a reference to a component.
	func get_component(component:Variant) -> Glecs.Component:
		return _get_component(component)

	## Removes component data from this entity.
	func remove_component(component: Variant) -> Glecs.Entity:
		_remove_component(component)
		return self

	## Deletes this entity from the ECS world.
	##
	## Note: this does not delete the [Glecs.Entity] object, which is
	## a reference counted reference to the entity.
	func delete() -> void:
		_delete()

	## Adds an other entity as a tag or relationship to this entity.
	func add_entity(tag: Variant) -> Entity:
		_add_entity(tag)
		return self

	## Returns true if this entity has the given tag or relationship.
	func has_entity(tag: Variant) -> bool:
		return _has_entity(tag)

	## Removes the given tag or relationship with this entity.
	func remove_entity(tag: Variant) -> Entity:
		_remove_entity(tag)
		return self

	## Returns the ID of this entity according to its world.
	func get_id() -> int:
		return _get_id()

	## Returns the name of this entity.
	func get_name() -> String:
		return _get_name()

	## Sets the name of this entity.
	func set_name(value: String) -> Entity:
		_set_name(value)
		return self

	## Adds to this entity a relationship between the two given entites.
	func add_relation(relation: Variant, with_entity: Variant) -> Entity:
		_add_relation(relation, with_entity)
		return self

	## Removes from this entity a relationship between the two given entites.
	func remove_relation(relation: Variant, with_entity: Variant) -> Entity:
		_remove_relation(relation, with_entity)
		return self

	## Returns true if this entity reference is valid.
	##
	## An entity reference is valid if the following are true:
	## - The ID of the reference is a real entity in the world.
	## - The entity is not deleted.
	## - The world is not delete.
	func is_valid() -> bool:
		return _is_valid()

	## Returns the world object this entity is in.
	func get_world() -> Glecs.World:
		return _get_world()


class World extends _GlecsBaseWorld:
	
	var PROCESS_PIPELINE:= id_from_variant(&"glecs.process"):
		set(_v): return
	var PHYSICS_PROCESS_PIPELINE:= id_from_variant(&"glecs.physics_process"):
		set(_v): return

	func new_event_listener(
		event:Variant,
	) -> GlecsSystemBuilder:
		return _new_event_listener(event)

	func id_from_variant(entity: Variant) -> int:
		return _id_from_variant(entity)

	func new_pipeline(
		identifier:Variant,
		additional_parameters:Array[Callable]=[],
	) -> void:
		_new_pipeline(identifier, additional_parameters)

	func new_system(pipeline: Variant = PROCESS_PIPELINE) -> GlecsSystemBuilder:
		return _new_system(pipeline)

	func new_entity(name:String, with_components:Array[Variant]=[]) -> Glecs.Entity:
		return _new_entity(name, with_components)


class WorldNode extends _GlecsBaseWorldNode:

	var PROCESS_PIPELINE:= id_from_variant(&"glecs.process"):
		set(_v): return
	var PHYSICS_PROCESS_PIPELINE:= id_from_variant(&"glecs.physics_process"):
		set(_v): return

	func _ready() -> void:
		new_pipeline(PROCESS_PIPELINE, [get_process_delta_time])
		new_pipeline(PHYSICS_PROCESS_PIPELINE, [get_physics_process_delta_time])

	func _process(delta: float) -> void:
		run_pipeline(PROCESS_PIPELINE, delta)

	func _physics_process(delta: float) -> void:
		run_pipeline(PHYSICS_PROCESS_PIPELINE, delta)

	func new_event_listener(
		event:Variant,
	) -> GlecsSystemBuilder:
		return _new_event_listener(event)

	func id_from_variant(entity: Variant) -> int:
		return _id_from_variant(entity)

	func new_pipeline(
		identifier:Variant,
		additional_parameters:Array[Callable]=[],
	) -> void:
		_new_pipeline(identifier, additional_parameters)

	func new_system(pipeline: Variant = PROCESS_PIPELINE) -> GlecsSystemBuilder:
		return _new_system(pipeline)

	func new_entity(name:String, with_components:Array[Variant]=[]) -> Glecs.Entity:
		return _new_entity(name, with_components)
