
class_name GlecsSystemBuilder extends _GlecsBaseSystemBuilder


func with(component: Variant, inout:=INOUT_MODE_DEFAULT) -> GlecsSystemBuilder:
	return _with(component, inout)


func without(component:Variant) -> GlecsSystemBuilder:
	return _without(component)


func or_with(component: Variant, inout:=INOUT_MODE_DEFAULT) -> GlecsSystemBuilder:
	return _or_with(component, inout)


func maybe_with(component: Variant, inout:=INOUT_MODE_DEFAULT) -> GlecsSystemBuilder:
	return _maybe_with(component, inout)


func all_from(entity: Variant) -> GlecsSystemBuilder:
	return _all_from(entity)


func any_from(entity: Variant) -> GlecsSystemBuilder:
	return _any_from(entity)


func none_from(entity: Variant) -> GlecsSystemBuilder:
	return _none_from(entity)


func for_each(callable: Callable) -> void:
	_for_each(callable)
