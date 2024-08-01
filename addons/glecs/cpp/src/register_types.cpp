
#include "register_types.h"

#include "world.h"
#include "entity.h"
#include "registerable_entity.h"
#include "component.h"

#include <gdextension_interface.h>
#include <godot_cpp/core/defs.hpp>
#include <godot_cpp/classes/engine.hpp>
#include <godot_cpp/godot.hpp>

// Note to self: If there are undefined symbols showing up as errors in Godot, it may be that symbol was not implemented. Double check the cpp file of the symbol to make sure its name and path are correct, because you will not get warnings or errors if they aren't.

using namespace godot;

void initialize_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
		return;
	}

	godot::ClassDB::register_class<GlWorld>();
	godot::ClassDB::register_class<GlEntity>();
	godot::ClassDB::register_class<GlRegisterableEntity>();
	godot::ClassDB::register_class<GlComponent>();

	Engine::get_singleton()->register_singleton("GlGlobalWorld", memnew(GlWorld));
}

void uninitialize_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
		return;
	}

	Engine::get_singleton()->unregister_singleton("GlGlobalWorld");
}

extern "C" {
// Initialization.
GDExtensionBool GDE_EXPORT library_init(GDExtensionInterfaceGetProcAddress p_get_proc_address, const GDExtensionClassLibraryPtr p_library, GDExtensionInitialization *r_initialization) {
	godot::GDExtensionBinding::InitObject init_obj(p_get_proc_address, p_library, r_initialization);

	init_obj.register_initializer(initialize_module);
	init_obj.register_terminator(uninitialize_module);
	init_obj.set_minimum_library_initialization_level(MODULE_INITIALIZATION_LEVEL_SCENE);

	return init_obj.init();
}
}