
class_name GEWorldNode extends _BaseGEWorld

func new_component(name:StringName, component:Script) -> void:
	super(name, component)

func new_entity(with_components:Array[Script] = []) -> void:
	super(with_components)

func new_system(callable: Callable) -> GESystemBuilder:
	assert(
		not _is_callable_error(callable),
		_callable_error_message(callable),
	)
	var builder:= _BaseGESystemBuilder._new_for_world(callable, self)
	builder.set_script(GESystemBuilder)
	return builder

func _is_callable_error(callable:Callable) -> bool:
	if not callable.get_object() is Script:
		return true
	if callable.is_valid():
		return false
	for method in callable.get_object().get_script_method_list():
		if method[&"name"] == callable.get_method():
			if method[&"flags"] & METHOD_FLAG_STATIC != METHOD_FLAG_STATIC:
				return true
			return false
	return true

func _callable_error_message(callable:Callable) -> String:
	if not callable.get_object() is Script:
		return "Couldn't create system from Callable. Function \"{0}\" in object \"{1}\" is not a static method. Only static methods can be used as systems." \
			.format([callable.get_method(), callable.get_object()])
	var script_path:String = callable.get_object().get_script_property_list()[0][&"hint_string"]
	if script_path.is_empty():
		script_path = callable.get_object().get_script_property_list()[0][&"name"]
	if callable.is_valid():
		return ""
	for method in callable.get_object().get_script_method_list():
		if method[&"name"] == callable.get_method():
			if method[&"flags"] & METHOD_FLAG_STATIC != METHOD_FLAG_STATIC:
				return "Couldn't create system from Callable. Function \"{0}\" in script \"{1}\" is not a static method. Only static methods can be used as systems." \
					.format([callable.get_method(), script_path])
			return ""
	
	return "Couldn't create system from Callable. Found no function \"{0}\" in script \"{1}\"." \
		.format([callable.get_method(), script_path])



