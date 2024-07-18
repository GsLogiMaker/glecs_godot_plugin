
## Texture2D

extends GlecsComponent

const Self:= preload("./texture2d.gd")
const CanvasItemComponent:= preload("./canvas_item.gd")
const Render:= preload("../rendering/render.gd")

static func _get_members(): return {
	texture = null,
}
func get_texture() -> Texture2D: return getc(&"texture")
func set_texture(v:Texture2D) -> void: return setc(&"texture", v)

static func _registered(w:GlecsWorldObject):
	# On Render-Texture set with CanvasItemComponent
	w.new_event_listener(Glecs.ON_SET) \
		.with(w.pair(Render, Self)) \
		.with(CanvasItemComponent, Glecs.INOUT_MODE_FILTER) \
		.for_each(func(ctexture:Self, item:CanvasItemComponent):
			var texture:= ctexture.get_texture()
			RenderingServer.canvas_item_add_texture_rect(
				item.get_rid(),
				Rect2(-texture.get_size() / 2, texture.get_size()),
				texture,
				)
			)
