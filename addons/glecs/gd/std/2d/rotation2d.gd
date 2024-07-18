
## A component that represents a rotaition in 2D space.

extends GlecsComponent

const Self:= preload("./rotation2d.gd")
const CanvasItemC:= preload("./canvas_item.gd")
const Position2DC:= preload("./position2d.gd")
const Scale2DC:= preload("./scale2d.gd")

static func _get_members() -> Dictionary: return {
	angle = 0.0,
}
func get_angle() -> float: return getc(&"angle")
func set_angle(v:float) -> void: return setc(&"angle", v)

static func _registered(w:GlecsWorldObject):
	# On Rotation2DC set, update visual transform of CanvasItemC
	w.new_event_listener(Glecs.ON_SET) \
		.with(CanvasItemC, Glecs.INOUT_MODE_FILTER) \
		.with(Self) \
		.maybe_with(Position2DC) \
		.maybe_with(Scale2DC) \
		.for_each(func(item:CanvasItemC, rot:Self, pos:Position2DC, scl:Scale2DC):
			item.update_transform_c(pos, rot, scl)
			)
