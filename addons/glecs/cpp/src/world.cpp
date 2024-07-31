
#include "world.h"

#include <flecs/flecs.h>
#include <godot_cpp/core/class_db.hpp>

using namespace godot;

void GFWorld::_bind_methods() {
}

GFWorld::GFWorld() {
	_raw = ecs_init();
}

GFWorld::~GFWorld() {
	ecs_fini(_raw);
}

ecs_world_t * GFWorld::raw() {
	return _raw;
}