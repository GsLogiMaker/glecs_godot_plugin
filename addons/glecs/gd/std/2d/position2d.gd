
## A component that represents a position in 2D space.

extends GlecsComponent

const Self:= preload("./position2d.gd")
const CanvasItemC:= preload("./canvas_item.gd")
const Rotation2DC:= preload("./rotation2d.gd")
const Scale2DC:= preload("./scale2d.gd")

static func _get_members() -> Dictionary: return {
	vec = Vector2(),
}
func get_vec() -> Vector2: return getc(&"vec")
func set_vec(v:Vector2) -> void: return setc(&"vec", v)
func get_x() -> float: return getc(&"vec").x
func set_x(v:float) -> void: return setc(&"vec", Vector2(v, get_y()))
func get_y() -> float: return getc(&"vec").y
func set_y(v:float) -> void: return setc(&"vec", Vector2(get_x(), v))

static func _registered(w:GlecsWorldObject):
	# On Position2D set, update visual transform of CanvasItemC
	w.new_event_listener(Glecs.ON_SET) \
		.with(CanvasItemC, Glecs.INOUT_MODE_FILTER) \
		.with(Self) \
		.maybe_with(Rotation2DC) \
		.maybe_with(Scale2DC) \
		.for_each(func(item:CanvasItemC, pos:Self, rot:Rotation2DC, scl:Scale2DC):
			item.update_transform_c(pos, rot, scl)
			)


