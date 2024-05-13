
extends GlecsComponent

const DEFINE:= {
	dict = TYPE_DICTIONARY,
}

func get_dict() -> Dictionary:
	return getc(&"dict")

func set_dict(v:Dictionary) -> void:
	return setc(&"dict", v)

class OtherClass extends GlecsComponent: pass
