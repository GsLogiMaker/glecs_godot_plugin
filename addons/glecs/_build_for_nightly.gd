
extends SceneTree

const NIGHTLY_NAME:= "GlecsNightly"

const GLECS:= "res://addons/glecs/"
const OLD_EXTENSION:= "glecs.gdextension"
const NEW_EXTENSION:= "glecs.gdextension.release"
const SOURCE:= "rust"
const CFG:= "plugin.cfg"
const THIS:= "_build_for_nightly.cfg"

func _init() -> void:
	var dir:= DirAccess.open(GLECS)

	if dir == null:
		push_error(error_string(DirAccess.get_open_error()))
		quit(1)
		return

	var has_error:= false
	if not dir.file_exists(OLD_EXTENSION):
		push_error(
			"Unexpected file structure. Failed to find \"%s/%s\"",
			GLECS,
			OLD_EXTENSION,
		)
		has_error = true
	if not dir.file_exists(NEW_EXTENSION):
		push_error(
			"Unexpected file structure. Failed to find \"%s/%s\"",
			GLECS,
			NEW_EXTENSION,
		)
		has_error = true
	if not dir.file_exists(SOURCE):
		push_error(
			"Unexpected file structure. Failed to find \"%s/%s\"",
			GLECS,
			SOURCE,
		)
		has_error = true
	if not dir.file_exists(CFG):
		push_error(
			"Unexpected file structure. Failed to find \"%s/%s\"",
			GLECS,
			CFG,
		)
		has_error = true
	if not dir.file_exists(THIS):
		push_error(
			"Unexpected file structure. Failed to find \"%s/%s\"",
			GLECS,
			THIS,
		)
		has_error = true

	if has_error:
		quit(1)
		return

	var err:= 0

	err = dir.remove(OLD_EXTENSION)
	prints("Remove %s" % OLD_EXTENSION)
	if err != OK:
		push_error("Error while deleting old extension:", error_string(err))
		quit(1)
		return

	err = dir.rename(NEW_EXTENSION, OLD_EXTENSION)
	prints("Rename %s to %s"% [OLD_EXTENSION, NEW_EXTENSION])
	if err != OK:
		push_error("Error while renaming new extension:", error_string(err))
		quit(1)
		return

	var plugin_cfg:= ConfigFile.new()

	err = plugin_cfg.load("%s/%s" % [GLECS, CFG])
	prints("Set plung.cfg")
	if err != OK:
		push_error("Error while loading plugin.cfg:", error_string(err))
		quit(1)
		return

	plugin_cfg.set_value("plugin", "name", NIGHTLY_NAME)
	plugin_cfg.set_value(
		"plugin",
		"version",
		plugin_cfg.get_value("plugin", "version")+"-nightly",
	)

	quit()





