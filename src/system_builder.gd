
class_name GESystemBuilder extends _BaseGESystemBuilder

func reads(component:Script) -> GESystemBuilder:
	self._term(component)
	return self

func writes(component:Script) -> GESystemBuilder:
	self._mut_term(component)
	return self

func build() -> GESystemBuilder:
	self._build(self, self._world)
	return self
