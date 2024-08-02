
#include "world.h"
#include "entity.h"
#include "component_builder.h"
#include "utils.h"

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

ecs_entity_t GlWorld::coerce_id(Variant value) {
	String string;
	godot::CharString str;
	Vector2i vec;
	Object* obj;
	switch (value.get_type()) {
		case Variant::INT:
			return value;
		case Variant::VECTOR2I:
			vec = Vector2i(value);
			return ecs_pair(vec.x, vec.y);
		case Variant::OBJECT:
			obj = value;
			if (obj == nullptr) {
				ERR(0,
					"Null objects can't be coerced to valid entity IDs"
				);
			}
			if (obj->is_class(GlEntity::get_class_static())) {
				return ((GlEntity*) obj)->get_id();
			}
			ERR(0,
				"Objects of type ",
				obj->get_class(),
				" can't be coerced to valid entity IDs"
			);
			break;
		case Variant::STRING:
		case Variant::STRING_NAME:
		case Variant::NODE_PATH:
			string = value;
			str = string.utf8();
			return ecs_lookup_path_w_sep(raw(), 0, str, "/", "/root/", false);
		default:
			break;
	};

	ERR(0,
		"Variants of type ",
		value.get_type_name(value.get_type()),
		" can't be coerced to valid entity IDs"
	);
}

void GlWorld::progress(double delta) {
	ecs_progress(raw(), delta);
}

static GlWorld* singleton() {
	Object* singleton = Engine::get_singleton()
		->get_singleton("GlGlobalWorld");
	return Object::cast_to<GlWorld>(singleton);
}

Ref<GlComponentBuilder> GlWorld::component_builder() {
	Ref<GlComponentBuilder> builder = memnew(GlComponentBuilder);
	builder->set_world(this);
	return builder;
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
	godot::ClassDB::bind_method(D_METHOD("component_builder"), &GlWorld::component_builder);
	godot::ClassDB::bind_method(D_METHOD("coerce_id", "entity"), &GlWorld::coerce_id);
	godot::ClassDB::bind_method(D_METHOD("start_rest_api"), &GlWorld::start_rest_api);
	godot::ClassDB::bind_method(D_METHOD("progress", "delta"), &GlWorld::progress);
}