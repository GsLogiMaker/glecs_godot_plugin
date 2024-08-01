
#include "component.h"

#include <stdint.h>
#include <flecs.h>
#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/variant/utility_functions.hpp>

#define ERR(return_value, ...) \
	UtilityFunctions::printerr(__VA_ARGS__); \
	UtilityFunctions::push_error(__VA_ARGS__); \
	return return_value;

using namespace godot;

GlComponent::GlComponent() {
}
GlComponent::~GlComponent() {
}

void* GlComponent::get_member_at(int offset) {
	ecs_world_t* raw = get_world()->raw();
	int8_t* bytes = (int8_t*) ecs_get_mut_id(raw, get_source_id(), get_id());
	return (void*) &bytes[offset];
}

Variant GlComponent::get_member(String member) {
	ecs_world_t* raw = get_world()->raw();

	// Get member data
	const EcsMember* member_data = get_member_data(member);
	if (member_data == nullptr) {
		// Member data is null. This should never happen.
		ERR(nullptr, 
			"Member data is null"
		);
	}
	void* member_ptr = get_member_at(member_data->offset);

	const EcsPrimitive* primi = ecs_get(raw, member_data->type, EcsPrimitive);
	if (primi == nullptr) {
		// Member type is not a primitive
		ERR(nullptr,
			"Member is not a primitive. Don't know what to do."
		);
	}

	// Return member 
	switch (primi->kind) {
		case ecs_primitive_kind_t::EcsBool: return *(bool*) member_ptr;
		case ecs_primitive_kind_t::EcsChar: return *(char*) member_ptr;
		case ecs_primitive_kind_t::EcsByte: return *(uint8_t*) member_ptr;
		case ecs_primitive_kind_t::EcsU8: return *(uint8_t*) member_ptr;
		case ecs_primitive_kind_t::EcsU16: return *(uint16_t*) member_ptr;
		case ecs_primitive_kind_t::EcsU32: return *(uint32_t*) member_ptr;
		case ecs_primitive_kind_t::EcsU64: return *(uint64_t*) member_ptr;
		case ecs_primitive_kind_t::EcsI8: return *(int8_t*) member_ptr;
		case ecs_primitive_kind_t::EcsI16: return *(int16_t*) member_ptr;
		case ecs_primitive_kind_t::EcsI32: return *(int32_t*) member_ptr;
		case ecs_primitive_kind_t::EcsI64: return *(int64_t*) member_ptr;
		case ecs_primitive_kind_t::EcsF32: return *(float*) member_ptr;
		case ecs_primitive_kind_t::EcsF64: return *(double*) member_ptr;
		case ecs_primitive_kind_t::EcsUPtr: ERR(nullptr, "Can't hanlde uptr");
		case ecs_primitive_kind_t::EcsIPtr: ERR(nullptr, "Can't hanlde iptr");
		case ecs_primitive_kind_t::EcsString: return (char*) member_ptr;
		case ecs_primitive_kind_t::EcsEntity: return *(ecs_entity_t*) member_ptr;
		case ecs_primitive_kind_t::EcsId: return *(ecs_entity_t*) member_ptr;
		default:
			ERR(nullptr, 
				"Unhandled type"
			);
	}

	ERR(nullptr, 
		"Unknown error"
	);
}

const EcsMember* GlComponent::get_member_data(String member) {
	ecs_world_t* raw = get_world()->raw();
	const char* c_str = member.utf8().get_data();

	// Get member ID
	ecs_entity_t member_id = ecs_lookup_child(raw, get_id(), c_str);

	if (member_id == 0) {
		ERR(nullptr, 
			"No member named \"",
			member,
			"\" found in component \"",
			ecs_get_name(raw, get_id()),
			"\""
		);
	}

	// Get member data
	const EcsMember* member_data = ecs_get(raw, member_id, EcsMember);

	return member_data;
}

Ref<GlEntity> GlComponent::get_source_entity() {
	return GlEntity::from(get_source_id(), get_world());
}

ecs_entity_t GlComponent::get_source_id() {
	return source_entity_id;
}

void GlComponent::set_source_id(ecs_entity_t id) {
	source_entity_id = id;
}

void GlComponent::_bind_methods() {
	godot::ClassDB::bind_method(D_METHOD("get_source_entity"), &GlComponent::get_source_entity);
	godot::ClassDB::bind_method(D_METHOD("get_source_id"), &GlComponent::get_source_id);
	godot::ClassDB::bind_method(D_METHOD("get_member", "member"), &GlComponent::get_member);
}

