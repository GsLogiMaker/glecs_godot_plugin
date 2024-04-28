
class_name Components extends RefCounted

class Positional2D extends GlecsComponent:
	var transform:= Transform2D.IDENTITY

class Motional2D extends GlecsComponent:
	var velocity:= Vector2.ZERO

class List extends GlecsComponent:
	var elements:= {}

static func move_position(positional:Positional2D, motional:Motional2D) -> void:
	positional.transform.origin += motional.velocity
