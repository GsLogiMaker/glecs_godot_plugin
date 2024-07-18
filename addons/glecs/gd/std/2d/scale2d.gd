
## A component that represents scale in 2D space.

extends GlecsComponent

const Self:= preload("./scale2d.gd")
const CanvasItemC:= preload("./canvas_item.gd")
const Position2DC:= preload("./position2d.gd")
const Rotation2DC:= preload("./rotation2d.gd")

static func _get_members() -> Dictionary: return {
	scale = Vector2.ONE,
}
func get_scale() -> Vector2: return getc(&"scale")
func set_scale(v:Vector2) -> void: return setc(&"scale", v)

static func _registered(w:GlecsWorldObject):
	# On Scale2DC set, update visual transform of CanvasItemC
	w.new_event_listener(Glecs.ON_SET) \
		.with(CanvasItemC, Glecs.INOUT_MODE_FILTER) \
		.with(Self) \
		.maybe_with(Position2DC) \
		.maybe_with(Rotation2DC) \
		.for_each(func(item:CanvasItemC, scal:Self, pos:Position2DC, rot:Rotation2DC):
			item.update_transform_c(pos, rot, scal)
			)
