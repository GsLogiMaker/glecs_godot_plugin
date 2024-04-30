
class_name GlecsComponent extends _GlecsComponent

static func _registered(world:GlecsWorld) -> void:
	pass

func copy_from_component(from_component: GlecsComponent) -> void:
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
	
