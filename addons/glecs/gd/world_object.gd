
class_name GlecsWorldObject extends _GlecsBaseWorld

var component_properties:= 0

func _init() -> void:
	component_properties = _GlecsComponents.define_raw(
		self,
		0,
		"ComponentProperties",
	)
	Glecs.PROCESS = id_from_variant("Glecs/process")
	Glecs.PHYSICS_PROCESS = id_from_variant("Glecs/physics_process")
	Glecs.ON_INIT = id_from_variant("Glecs/OnInit")
	Glecs.ON_SET = id_from_variant("Glecs/OnSet")
	
	register(Glecs.Std, "std")

func get_child(path: String) -> GlecsEntity:
	return GlecsEntity.from(_GlecsBindings.lookup(self, path), self)

func new_event_listener(
	event:Variant,
) -> GlecsSystemBuilder:
	return _new_event_listener(event)

## Converts a [Variant] to an entity ID. [br] [br]
##
## How Variants are converts: [br]
## - [int]: No change. [br]
## - [float]: Converted to int. [br]
## - [Vector2i]: Interpreted as a pair. [br]
## - [Vector2]: Converted to integers, then interpreted as a pair. [br]
## - [String]: Finds entity by its name. [br]
## - [StringName]: Finds entity by its name. [br]
## - [GlecsEntity]: Calls [method GlecsEntity.get_id]. [br]
## - [GlecsComponent]: Throws exception for being too ambiguous. (Should
## 		it return the component type ID, or the ID of the
## 		entity its attached to?) [br]
## - [Script] extending [GlecsEntity]: Returns the ID registered 
## 		with the world. [br]
## - [Script] extending [GlecsComponent]: Returns the ID registered 
## 		with the world. [br]
## - Everything else: Throws an exception.
func id_from_variant(entity: Variant) -> int:
	return _id_from_variant(entity)

func new_pipeline(
	identifier:Variant,
	additional_parameters:Array[Callable]=[],
) -> void:
	_new_pipeline(identifier, additional_parameters)

func register(module:Script, name: String = "") -> void:
	_register_script(module, name)

func new_system(pipeline: Variant = Glecs.PROCESS) -> GlecsSystemBuilder:
	return _new_system(pipeline)

