
class_name GESystemBuilder extends _BaseGESystemBuilder

func reads(component:Script) -> GESystemBuilder:
	_reads(component)
	return self

func writes(component:Script) -> GESystemBuilder:
	_writes(component)
	return self

func build() -> void:
	_build(self, self._world)
