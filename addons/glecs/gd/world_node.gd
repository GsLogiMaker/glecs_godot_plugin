
class_name GlecsWorldNode extends _GlecsWorldNode

## Emitted when a component is added to an entity, before the entity gets set.
## TODO: Handle component parameters being unitialized when accessed through an OnAdd observer
const EVENT_ON_ADD:= &"on_add"
## Emitted after a component's value changed.
const EVENT_ON_SET:= &"on_set"

const PIPELINE_PROCESS:= &"process"
const PIPELINE_PHYSICS_PROCESS:= &"physics_process"

func new_event_listener(
	event:Variant,
) -> GlecsSystemBuilder:
	return _new_event_listener(event)

func new_pipeline(
	identifier:Variant,
	additional_parameters:Array[Callable]=[],
) -> void:
	_new_pipeline(identifier, additional_parameters)

func new_system(pipeline: Variant = PIPELINE_PROCESS) -> GlecsSystemBuilder:
	return _new_system(pipeline)

func new_entity(name:String, with_components:Array[Variant]=[]) -> GlecsEntity:
	return _new_entity(name, with_components)
	
