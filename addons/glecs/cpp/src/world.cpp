
#include "world.h"
#include <flecs/flecs.h>
#include <godot_cpp/core/class_db.hpp>

using namespace godot;

void GFWorld::_bind_methods() {
}

GFWorld::GFWorld() {
	raw = ecs_init();
}

GFWorld::~GFWorld() {
	ecs_fini(raw);
}