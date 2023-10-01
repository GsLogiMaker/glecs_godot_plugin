
class_name GEComponentAccess extends _BaseGEComponentAccess

func _get(property:StringName) -> Variant:
	prints(property, 1)
	prints(property, 1.5, _component_has(property))
	assert(
		_component_has(property),
		"Component {name} has no property by name '{property}'" \
			.format({name="[name]", property=property}),
	)
	prints(property, 2)
	var value = _component_get(property)
	prints(property, 3)
	return value

func _set(property:StringName, value:Variant) -> bool:
	var err:= _component_set(property, value)
	assert(err == OK, error_msg(err, property, value))
	return err == OK

func _get_property_list() -> Array[Dictionary]:
	return _component_get_property_list()

func error_msg(err:int, property:StringName, value:Variant) -> String:
	var name:= "[name]"
	match err:
		ERR_DOES_NOT_EXIST:
			return "Component {name} has no property by name '{property}'" \
				.format({name=name, property=property})
		ERR_DATABASE_CANT_WRITE:
			return "Can't write to {name}.{property} because {name} was queried as readable, not writable." \
				.format({name=name, property=property})
				
	return "Err message not implemented"
