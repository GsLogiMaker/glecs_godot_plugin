
#include "world.h"

#include <flecs.h>
#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/classes/engine.hpp>

using namespace godot;

GlWorld::GlWorld() {
	_raw = ecs_init();
}

GlWorld::~GlWorld() {
	ecs_fini(_raw);
}

static GlWorld* singleton() {
	Object* singleton = Engine::get_singleton()
		->get_singleton("GlGlobalWorld");
	return Object::cast_to<GlWorld>(singleton);
}

ecs_world_t * GlWorld::raw() {
	return _raw;
}

void GlWorld::_bind_methods() {
}