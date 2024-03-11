
class_name GEWorldNode extends _BaseGEWorld

const EVENT_ON_ADD:= &"on_add"
const EVENT_ON_SET:= &"on_set"

const PIPELINE_PROCESS:= &"process"
const PIPELINE_PHYSICS_PROCESS:= &"physics_process"

func new_event_listener(
	event:Variant,
) -> SystemBuilder:
	return _new_event_listener(event)

func new_pipeline(
	identifier:Variant,
	additional_parameters:Array[Callable]=[],
) -> void:
	_new_pipeline(identifier, additional_parameters)

func new_system(pipeline: Variant = PIPELINE_PROCESS) -> SystemBuilder:
	return _new_system(pipeline)

func new_entity(name:String, with_components:Array[Script]=[]) -> Entity:
	return _new_entity(name, with_components)
	
