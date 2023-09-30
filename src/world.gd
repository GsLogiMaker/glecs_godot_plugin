
class_name GEWorldNode extends _BaseGEWorld

func new_system(callable: Callable) -> GESystemBuilder:
	var builder:= _BaseGESystemBuilder._new_for_world(callable, self)
	builder.set_script(GESystemBuilder)
	return builder

func new_entity(with_components:Array[Script]) -> void:
	self._new_entity(with_components)

func register_component(name:StringName, component:Script) -> void:
	# Godot Rust can't convert String to StringName automaticly, so this
	# wrapper exists to do that.
	super(name, component)

