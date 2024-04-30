
class_name GlecsWorld extends _GlecsWorld

## Emitted when a component is added to an entity, before the entity gets set.
var ON_ADD_EVENT:= id_from_variant(&"flecs.on_add"):
	set(_v): return
## Emitted after a component's value changed.
var ON_SET_EVENT:= id_from_variant(&"flecs.on_set"):
	set(_v): return
var PREFAB_TAG:= id_from_variant(&"flecs.prefab"):
	set(_v): return

var PROCESS_PIPELINE:= id_from_variant(&"glecs.process"):
	set(_v): return
var PHYSICS_PROCESS_PIPELINE:= id_from_variant(&"glecs.physics_process"):
	set(_v): return

func new_event_listener(
	event:Variant,
) -> GlecsSystemBuilder:
	return _new_event_listener(event)

func id_from_variant(entity: Variant) -> int:
	return _id_from_variant(entity)

func new_pipeline(
	identifier:Variant,
	additional_parameters:Array[Callable]=[],
) -> void:
	_new_pipeline(identifier, additional_parameters)

func new_system(pipeline: Variant = PROCESS_PIPELINE) -> GlecsSystemBuilder:
	return _new_system(pipeline)

func new_entity(name:String, with_components:Array[Variant]=[]) -> GlecsEntity:
	return _new_entity(name, with_components)
	
