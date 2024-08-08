

#include "component_builder.h"
#include "world.h"
#include "utils.h"

#include <stdlib.h>
#include <flecs.h>

using namespace godot;

GlComponentBuilder::GlComponentBuilder() {
	component_desc = {0};
	struct_desc = {0};
	member_names = Array();
	world = {0};
	built = {0};
}
GlComponentBuilder::~GlComponentBuilder() {
}

Ref<GlComponentBuilder> GlComponentBuilder::add_member(
	String member,
	Variant::Type type
) {
	const char* ERR_ADD_COMPONENT = "Failed to add member to component builder\n";

	if (get_member_count() == ECS_MEMBER_DESC_CACHE_SIZE) {
		ERR(Ref(this),
			ERR_ADD_COMPONENT,
			"Max member count reached"
		);
	}

	EntityResult ecs_type_result = Utils::variant_type_to_id(type);
	if (!ecs_type_result.is_ok()) {
		ERR(Ref(this),
			ERR_ADD_COMPONENT,
			ecs_type_result.unwrap_err()
		);
	}
	ecs_entity_t ecs_type = ecs_type_result.unwrap();

	struct_desc.members[get_member_count()] = {
		.type = ecs_type
	};
	member_names.append(member);

	return Ref(this);
}

int GlComponentBuilder::get_member_count() {
	return member_names.size();
}

Ref<GlComponentBuilder> GlComponentBuilder::set_name(
	String name_
) {
	name = name_;
	return Ref(this);
}

void GlComponentBuilder::build() {
	const char* FAILED_TO_BUILD = "Failed to build component\n";
	if (built) {
		ERR(/**/,
			FAILED_TO_BUILD,
			"Component builder was already built"
		);
	}
	built = true;

	// Set names to temporary pointers
	CharString name_utf8 = name.utf8();
	CharString member_names_utf8[ECS_MEMBER_DESC_CACHE_SIZE] = {0};
	component_desc.type.name = name_utf8.ptr();
	for (int i=0; i != get_member_count(); i++) {
		member_names_utf8[i] = String(member_names[i]).utf8();
		struct_desc.members[i].name = member_names_utf8[i].ptr();
	}

	ecs_world_t* raw = world->raw();

	// Create component entity
	ecs_entity_t component_id = ecs_new(raw);
	component_desc.entity = component_id;
	struct_desc.entity = component_id;

	ecs_entity_t struct_id = ecs_struct_init(raw, &struct_desc);

	ecs_type_hooks_t hooks = {
		.ctor = GlComponentBuilder::ctor,
		.binding_ctx = new HooksBindingContext(world),
		.binding_ctx_free = [](void* ptr) {
			HooksBindingContext* ctx = (HooksBindingContext*)ptr;
			delete ctx;
		}
	}; ecs_set_hooks_id(raw, component_id, &hooks);

	ecs_add_path(raw, component_id, 0, component_desc.type.name);
}

void GlComponentBuilder::set_world(GlWorld* world_) {
	world = world_;
}

// **********************************************
// *** PROTECTED ***
// **********************************************

void GlComponentBuilder::_bind_methods() {
	godot::ClassDB::bind_method(D_METHOD("add_member", "member", "type"), &GlComponentBuilder::add_member);
	godot::ClassDB::bind_method(D_METHOD("set_name", "name"), &GlComponentBuilder::set_name);
	godot::ClassDB::bind_method(D_METHOD("build"), &GlComponentBuilder::build);

}

// **********************************************
// *** PRIVATE ***
// **********************************************

void GlComponentBuilder::ctor(void* ptr, int32_t count, const ecs_type_info_t* type_info) {
	uint8_t* list = (uint8_t*)ptr;
	HooksBindingContext* ctx = (HooksBindingContext*) type_info->hooks.binding_ctx;

	for (int i=0; i != count; i++) {
		uint8_t* item = &list[i*type_info->size];
		ctx->world->init_component_ptr((void*)item, type_info->component, Variant());
	}
}

HooksBindingContext::HooksBindingContext(GlWorld* world_) {
	world = world_;
}
HooksBindingContext::~HooksBindingContext() {
}
