
class_name Components extends RefCounted

class Positional2D extends Glecs.Component:
	var transform:= Transform2D.IDENTITY

class Motional2D extends Glecs.Component:
	var velocity:= Vector2.ZERO

class List extends Glecs.Component:
	var elements:= {}

static func move_position(positional:Positional2D, motional:Motional2D) -> void:
	positional.transform.origin += motional.velocity
