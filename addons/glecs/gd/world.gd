
class_name GEWorldNode extends _BaseGEWorld

func new_event_listener(
	event:Variant,
	terms:Array[Script],
	callable:Callable,
) -> void:
	_new_event_listener(event, terms, callable)

func new_pipeline(
	identifier:Variant,
	additional_parameters:Array[Callable]=[],
) -> void:
	_new_pipeline(identifier, additional_parameters)

func add_system(
	terms:Array[Script],
	callable:Callable,
	pipeline:Variant="process",
) -> void:
	_add_system(terms, callable, pipeline)

func new_process_system(
	terms:Array[Script],
	callable:Callable,
) -> void:
	_new_process_system(terms, callable)

func new_entity(name:String, with_components:Array[Script] = []) -> Entity:
	return _new_entity(name, with_components)
	
