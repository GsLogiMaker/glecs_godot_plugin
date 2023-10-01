
class_name GEComponent extends _BaseGEComponent

func _get(property:StringName) -> Variant:
	return _component_get(property)

func _set(property:StringName, value:Variant) -> bool:
	return _component_set(property, value)

func _get_property_list() -> Array[Dictionary]:
	return _component_get_property_list()
