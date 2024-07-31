
#include "world.h"

#include <flecs.h>
#include <godot_cpp/core/class_db.hpp>

using namespace godot;

void GlWorld::_bind_methods() {
}

GlWorld::GlWorld() {
	_raw = ecs_init();
}

GlWorld::~GlWorld() {
	ecs_fini(_raw);
}

ecs_world_t * GlWorld::raw() {
	return _raw;
}