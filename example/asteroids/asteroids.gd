
extends Node2D

@onready var world:Glecs.WorldNode = $Glecs.WorldNode

var texture:= load("res://icon.png")

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	var e:= world.new_entity("Test", [])
	e.add_component(CompTexture2D, [texture])
	
	#prints(e.get_component(RenderableSprite2D).get_rid())

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass
 
class RenderableSprite2D extends Glecs.Component:
	const _VAR_rid:RID = RID()
	
	static func _registered(world:Glecs.World):
		# Initialization
		world.new_event_listener(Glecs.WorldNode.EVENT_ON_ADD) \
			.with(RenderableSprite2D) \
			.for_each(func(sprite:RenderableSprite2D):
				sprite.set_rid(RenderingServer.canvas_item_create())
				RenderingServer.canvas_item_set_parent(
					sprite.get_rid(),
					(Engine.get_main_loop() as SceneTree)
						.current_scene
						.get_viewport()
						.world_2d.canvas
				)
				)
		
		# Set texture
		world.new_event_listener(Glecs.WorldNode.EVENT_ON_ADD) \
			.with(RenderableSprite2D) \
			.with(CompTexture2D) \
			.for_each(func(sprite:RenderableSprite2D, c_texture:CompTexture2D):
				if c_texture.get_texture() == null:
					return
				RenderingServer.canvas_item_add_texture_rect(
					sprite.get_rid(),
					Rect2(-c_texture.get_texture().get_size() / 2, c_texture.get_texture().get_size()),
					c_texture.get_texture(),
				)
				)
	
	func get_rid() -> RID:
		return getc(&"rid")
	func set_rid(v:RID) -> void:
		setc(&"rid", v)

class CompTexture2D extends Glecs.Component:
	const _VAR_texture:Texture2D = null
	
	func get_texture() -> Texture2D:
		var x = getc(&"texture")
		return x
	func set_texture(v:Texture2D) -> void:
		setc(&"texture", v)
	
	static func _registered(world:Glecs.World):
		world.new_event_listener(world.EVENT_ON_ADD) \
			.with(CompTexture2D) \
			.for_each(func(c:CompTexture2D):
				prints(c.get_texture())
				)
			
