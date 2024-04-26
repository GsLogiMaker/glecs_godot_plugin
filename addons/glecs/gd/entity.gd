
class_name Entity extends _BaseGEEntity

func add_component(component:Variant, data:Variant=null) -> void:
	_add_component(component, data)

func get_component(component:Variant) -> GEComponent:
	return _get_component(component)

func remove_component(component: Variant) -> void:
	_remove_component(component)

func get_name() -> String:
	return _get_name()

func set_name(value: String) -> void:
	_set_name(value)

func add_relation(relation: Variant, with_entity: Entity) -> void:
	_add_relation(relation, with_entity)

func remove_relation(relation: Variant, with_entity: Entity) -> void:
	_remove_relation(relation, with_entity)
