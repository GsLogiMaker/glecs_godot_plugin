
class_name GEWorldNode extends _BaseGEWorld

func new_component(name:StringName, component:Script) -> void:
	super(name, component)

func new_entity(with_components:Array[Script] = []) -> void:
	super(with_components)

func new_system(callable: Callable) -> GESystemBuilder:
	var builder:= _BaseGESystemBuilder._new_for_world(callable, self)
	builder.set_script(GESystemBuilder)
	return builder





