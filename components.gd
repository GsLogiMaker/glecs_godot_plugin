
class_name Components extends RefCounted

class Positional2D extends GEComponent:
	var transform:= Transform2D.IDENTITY

class Motional2D extends GEComponent:
	var velocity:= Vector2.ZERO

class List extends GEComponent:
	var elements:= {}

static func move_position(positional:Positional2D, motional:Motional2D) -> void:
	positional.transform.origin += motional.velocity
	prints(positional.transform.origin)
