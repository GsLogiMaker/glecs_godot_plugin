
class_name GEComponent extends _BaseGEComponent

func _get(property:StringName) -> Variant:
	return _component_get(property)

func _set(property:StringName, value:Variant) -> bool:
	return _component_set(property, value)

func _get_property_list() -> Array[Dictionary]:
	return [{
		name=&"run_speed",
		type = TYPE_FLOAT,
		usage=PROPERTY_USAGE_EDITOR,
	}]
#	return _component_get_property_list(property, value)
