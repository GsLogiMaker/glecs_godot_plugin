
class_name GlecsWorldNode extends _GlecsBaseWorldNode

func _process(delta: float) -> void:
	run_pipeline(Glecs.PROCESS, delta)

func _physics_process(delta: float) -> void:
	run_pipeline(Glecs.PHYSICS_PROCESS, delta)

func new_event_listener(
	event:Variant,
) -> GlecsSystemBuilder:
	return _new_event_listener(event)

func id_from_variant(entity: Variant) -> int:
	return _id_from_variant(entity)

func new_pipeline(
	name: String,
	additional_parameters:Array[Callable]=[],
) -> GlecsEntity:
	return _new_pipeline(name, additional_parameters)

func new_system(pipeline: Variant = Glecs.PROCESS) -> GlecsSystemBuilder:
	return _new_system(pipeline)

func new_entity(name:String, with_components:Array[Variant]=[]) -> GlecsEntity:
	return _new_entity(name, with_components)
