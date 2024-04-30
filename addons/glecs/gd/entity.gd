
class_name GlecsEntity extends _GlecsEntity

static func _registered(world:GlecsWorld) -> void:
	pass

## Creates a new entity
static func spawn(world:GlecsWorld = null) -> GlecsEntity:
	return _GlecsEntity._spawn(world)

## Creates a reference to an existing entity from the given [Varaint].
static func from(entity:Variant, world:GlecsWorld = null) -> GlecsEntity:
	return _GlecsEntity._from(entity, world)

func add_component(component:Variant, data:Variant=null) -> void:
	_add_component(component, data)

func get_component(component:Variant) -> GlecsComponent:
	return _get_component(component)

func remove_component(component: Variant) -> void:
	_remove_component(component)

func delete() -> void:
	_delete()

func add_entity(entity: Variant) -> void:
	_add_entity(entity)

func has_entity(entity: Variant) -> bool:
	return _has_entity(entity)

func remove_entity(entity: Variant) -> void:
	_remove_entity(entity)

func get_id() -> int:
	return _get_id()

func get_name() -> String:
	return _get_name()

func set_name(value: String) -> void:
	_set_name(value)

func add_relation(relation: Variant, with_entity: GlecsEntity) -> void:
	_add_relation(relation, with_entity)

func remove_relation(relation: Variant, with_entity: GlecsEntity) -> void:
	_remove_relation(relation, with_entity)

func is_valid() -> bool:
	return _is_valid()

func get_world() -> _GlecsWorld:
	return _get_world()
