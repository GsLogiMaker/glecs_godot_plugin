
class_name GESystemBuilder extends _BaseGESystemBuilder

func reads(component:Script) -> GESystemBuilder:
	_term(component)
	return self

func writes(component:Script) -> GESystemBuilder:
	_mut_term(component)
	return self

func build() -> GESystemBuilder:
	_build(self, self._world)
	return self
