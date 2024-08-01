
#include "world.h"

#include <flecs.h>
#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/classes/engine.hpp>
#include <godot_cpp/variant/string_name.hpp>

using namespace godot;

GlWorld::GlWorld() {
	_raw = ecs_init();
	ECS_IMPORT(raw(), FlecsStats);
}

GlWorld::~GlWorld() {
	ecs_fini(_raw);
}

void GlWorld::progress(double delta) {
	ecs_progress(raw(), delta);
}

static GlWorld* singleton() {
	Object* singleton = Engine::get_singleton()
		->get_singleton("GlGlobalWorld");
	return Object::cast_to<GlWorld>(singleton);
}

void GlWorld::start_rest_api() {
	ecs_entity_t rest_id = ecs_lookup_path_w_sep(raw(), 0, "flecs.rest.Rest", ".", "", false);
	EcsRest rest = (EcsRest)EcsRest();
	ecs_set_id(raw(), rest_id, rest_id, sizeof(EcsRest), &rest);
}

ecs_world_t * GlWorld::raw() {
	return _raw;
}

void GlWorld::_bind_methods() {
	godot::ClassDB::bind_method(D_METHOD("start_rest_api"), &GlWorld::start_rest_api);
	godot::ClassDB::bind_method(D_METHOD("progress", "delta"), &GlWorld::progress);
}