
#include "component.h"
#include "godot_cpp/variant/variant.hpp"
#include "utils.h"

#include <stdint.h>
#include <flecs.h>
#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/variant/utility_functions.hpp>

using namespace godot;

GlComponent::GlComponent() {
}
GlComponent::~GlComponent() {
}

void GlComponent::set_member(String member, Variant value) {
	ecs_world_t* raw = get_world()->raw();

	// Get member data
	const EcsMember* member_data = get_member_data(member);
	if (member_data == nullptr) {
		// Member data is null. This should never happen.
		ERR(/**/,
			"Member metadata is null"
		);
	}
	void* member_ptr = get_member_ptr_mut_at(member_data->offset);
	if (member_ptr == nullptr) {
		ERR(/**/,
			"Member pointer is null"
		);
	}

	// Return member
	set_member_value_as_type(member_ptr, value, member_data->type);
}

void GlComponent::set_member_value_as_primitive(
	void* ptr,
	Variant value,
	ecs_primitive_kind_t primitive
) {
	int8_t value_char;

	#define EXPECT_VARIANT(VALUE, VARIANT_TYPE) \
		VoidResult r = Utils::check_variant_matches(VALUE, VARIANT_TYPE); \
		if (!r.is_ok()) { ERR(/**/, \
			r.unwrap_err() \
		); } \

	#define SET_MEMBER(variant_type, real_type) \
		{ \
			EXPECT_VARIANT(value, Variant::variant_type); \
			*(real_type*) ptr = value; \
		}
	#define SET_MEMBER_CHAR() \
		{ \
			EXPECT_VARIANT(value, Variant::INT); \
			uint8_t intermediate = value; \
			*(char*) ptr = intermediate; \
		}
	#define SET_MEMBER_ETT() \
		{ \
			ecs_entity_t id = get_world()->coerce_id(value); \
			*(ecs_entity_t*) ptr = id; \
		}

	switch (primitive) {
		case ecs_primitive_kind_t::EcsBool: SET_MEMBER(BOOL, bool); break;
		case ecs_primitive_kind_t::EcsByte: SET_MEMBER(INT, uint8_t); break;
		case ecs_primitive_kind_t::EcsU8: SET_MEMBER(INT, uint8_t); break;
		case ecs_primitive_kind_t::EcsU16: SET_MEMBER(INT, uint16_t); break;
		case ecs_primitive_kind_t::EcsU32: SET_MEMBER(INT, uint32_t); break;
		case ecs_primitive_kind_t::EcsU64: SET_MEMBER(INT, uint64_t); break;
		case ecs_primitive_kind_t::EcsI8: SET_MEMBER(INT, int8_t); break;
		case ecs_primitive_kind_t::EcsI16: SET_MEMBER(INT, int16_t); break;
		case ecs_primitive_kind_t::EcsI32: SET_MEMBER(INT, int32_t); break;
		case ecs_primitive_kind_t::EcsI64: SET_MEMBER(INT, int64_t); break;
		case ecs_primitive_kind_t::EcsF32: SET_MEMBER(FLOAT, float); break;
		case ecs_primitive_kind_t::EcsF64: SET_MEMBER(FLOAT, double); break;
		case ecs_primitive_kind_t::EcsChar: SET_MEMBER_CHAR(); break;
		case ecs_primitive_kind_t::EcsString: ERR(/**/, "TODO: Concerned about memory management");
		case ecs_primitive_kind_t::EcsEntity: SET_MEMBER_ETT(); break;
		case ecs_primitive_kind_t::EcsId: SET_MEMBER_ETT(); break;
		case ecs_primitive_kind_t::EcsUPtr: ERR(/**/, "Can't hanlde uptr");
		case ecs_primitive_kind_t::EcsIPtr: ERR(/**/, "Can't hanlde iptr");
		default:
			ERR(/**/,
				"Unhandled type"
			);
	}

	#undef SET_MEMBER
	#undef SET_MEMBER_CHAR
	#undef SET_MEMBER_ETT
}

void GlComponent::set_member_value_as_type(
	void* ptr,
	Variant value,
	ecs_entity_t type
) {
	ecs_world_t* raw = get_world()->raw();

	#define SET_MEMBER(variant_type, real_type) \
		EXPECT_VARIANT(value, Variant::variant_type); \
		*(real_type*) ptr = value;

	if (type == GlWorld::glecs_meta_real) { SET_MEMBER(FLOAT, real_t); return; }
	else if (type == GlWorld::glecs_meta_vector2) { SET_MEMBER(VECTOR2, Vector2); return; }
	else if (type == GlWorld::glecs_meta_vector2i) { SET_MEMBER(VECTOR2I, Vector2i); return; }
	else if (type == GlWorld::glecs_meta_rect2) { SET_MEMBER(RECT2, Rect2); return; }
	else if (type == GlWorld::glecs_meta_rect2i) { SET_MEMBER(RECT2I, Rect2i); return; }
	else if (type == GlWorld::glecs_meta_vector3) { SET_MEMBER(VECTOR3, Vector3); return; }
	else if (type == GlWorld::glecs_meta_vector3i) { SET_MEMBER(VECTOR3I, Vector3i); return; }
	else if (type == GlWorld::glecs_meta_transform2d) { SET_MEMBER(TRANSFORM2D, Transform2D); return; }
	else if (type == GlWorld::glecs_meta_vector4) { SET_MEMBER(VECTOR4, Vector4); return; }
	else if (type == GlWorld::glecs_meta_vector4i) { SET_MEMBER(VECTOR4I, Vector4i); return; }
	else if (type == GlWorld::glecs_meta_plane) { SET_MEMBER(PLANE, Plane); return; }
	else if (type == GlWorld::glecs_meta_quaternion) { SET_MEMBER(QUATERNION, Quaternion); return; }
	else if (type == GlWorld::glecs_meta_aabb) { SET_MEMBER(AABB, AABB); return; }
	else if (type == GlWorld::glecs_meta_basis) { SET_MEMBER(BASIS, Basis); return; }
	else if (type == GlWorld::glecs_meta_transform3d) { SET_MEMBER(TRANSFORM3D, Transform3D); return; }
	else if (type == GlWorld::glecs_meta_projection) { SET_MEMBER(PROJECTION, Projection); return; }
	else if (type == GlWorld::glecs_meta_color) { SET_MEMBER(COLOR, Color); return; }

	else if (type == GlWorld::glecs_meta_string_name) { SET_MEMBER(STRING_NAME, StringName); return; }
	else if (type == GlWorld::glecs_meta_node_path) { SET_MEMBER(NODE_PATH, NodePath); return; }
	else if (type == GlWorld::glecs_meta_rid) { SET_MEMBER(RID, RID); return; }
	else if (type == GlWorld::glecs_meta_object) { SET_MEMBER(OBJECT, Variant); return; }
	else if (type == GlWorld::glecs_meta_callable) { SET_MEMBER(CALLABLE, Callable); return; }
	else if (type == GlWorld::glecs_meta_signal) { SET_MEMBER(SIGNAL, Signal); return; }
	else if (type == GlWorld::glecs_meta_dictionary) { SET_MEMBER(DICTIONARY, Variant); return; }
	else if (type == GlWorld::glecs_meta_array) { SET_MEMBER(ARRAY, Variant); return; }
	else if (type == GlWorld::glecs_meta_packed_int32_array) { SET_MEMBER(PACKED_INT32_ARRAY, PackedInt32Array); return; }
	else if (type == GlWorld::glecs_meta_packed_int64_array) { SET_MEMBER(PACKED_INT64_ARRAY, PackedInt64Array); return; }
	else if (type == GlWorld::glecs_meta_packed_float32_array) { SET_MEMBER(PACKED_FLOAT32_ARRAY, PackedFloat32Array); return; }
	else if (type == GlWorld::glecs_meta_packed_float64_array) { SET_MEMBER(PACKED_FLOAT64_ARRAY, PackedFloat64Array); return; }
	else if (type == GlWorld::glecs_meta_packed_string_array) { SET_MEMBER(PACKED_STRING_ARRAY, PackedStringArray); return; }
	else if (type == GlWorld::glecs_meta_packed_vector2_array) { SET_MEMBER(PACKED_VECTOR2_ARRAY, PackedVector2Array); return; }
	else if (type == GlWorld::glecs_meta_packed_vector3_array) { SET_MEMBER(PACKED_VECTOR3_ARRAY, PackedVector3Array); return; }
	else if (type == GlWorld::glecs_meta_packed_color_array) { SET_MEMBER(PACKED_COLOR_ARRAY, PackedColorArray); return; }

	if (ecs_has_id(raw, type, ecs_id(EcsPrimitive))) {
		return set_member_value_as_primitive(
			ptr,
			value,
			ecs_get(raw, type, EcsPrimitive)->kind
		);
	}

	ERR(/**/,
		"Can't convert type ", ecs_get_name(raw, type), " to Variant"
	);

	#undef SET_MEMBER
}

Variant GlComponent::get_member(String member) {
	ecs_world_t* raw = get_world()->raw();

	// Get member data
	const EcsMember* member_data = get_member_data(member);
	if (member_data == nullptr) {
		// Member data is null. This should never happen.
		ERR(nullptr,
			"Member metadata is null"
		);
	}
	void* member_value_ptr = get_member_ptr_mut_at(member_data->offset);
	if (member_value_ptr == nullptr) {
		ERR(nullptr,
			"Member value is null"
		);
	}


	return member_value_as_type(member_value_ptr, member_data->type);
}

void* GlComponent::get_member_ptr_mut_at(int offset) {
	ecs_world_t* raw = get_world()->raw();
	int8_t* bytes = (int8_t*) ecs_get_mut_id(raw, get_source_id(), get_id());
	return (void*) &bytes[offset];
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

Variant GlComponent::member_value_as_primitive(
	void* ptr,
	ecs_primitive_kind_t primitive
) {
	switch (primitive) {
		case ecs_primitive_kind_t::EcsBool: return *(bool*) ptr;
		case ecs_primitive_kind_t::EcsChar: return *(char*) ptr;
		case ecs_primitive_kind_t::EcsByte: return *(uint8_t*) ptr;
		case ecs_primitive_kind_t::EcsU8: return *(uint8_t*) ptr;
		case ecs_primitive_kind_t::EcsU16: return *(uint16_t*) ptr;
		case ecs_primitive_kind_t::EcsU32: return *(uint32_t*) ptr;
		case ecs_primitive_kind_t::EcsU64: return *(uint64_t*) ptr;
		case ecs_primitive_kind_t::EcsI8: return *(int8_t*) ptr;
		case ecs_primitive_kind_t::EcsI16: return *(int16_t*) ptr;
		case ecs_primitive_kind_t::EcsI32: return *(int32_t*) ptr;
		case ecs_primitive_kind_t::EcsI64: return *(int64_t*) ptr;
		case ecs_primitive_kind_t::EcsF32: return *(float*) ptr;
		case ecs_primitive_kind_t::EcsF64: return *(double*) ptr;
		case ecs_primitive_kind_t::EcsUPtr: ERR(nullptr, "Can't hanlde uptr");
		case ecs_primitive_kind_t::EcsIPtr: ERR(nullptr, "Can't hanlde iptr");
		case ecs_primitive_kind_t::EcsString: return *(char**) ptr;
		case ecs_primitive_kind_t::EcsEntity: return *(ecs_entity_t*) ptr;
		case ecs_primitive_kind_t::EcsId: return *(ecs_entity_t*) ptr;
		default:
			ERR(nullptr,
				"Unknown primitive type"
			);
	}
}

Variant GlComponent::member_value_as_type(
	void* ptr,
	ecs_entity_t type
) {
	ecs_world_t* raw = get_world()->raw();

	if (type == GlWorld::glecs_meta_real) { return Variant( *(float*) ptr );}
	else if (type == GlWorld::glecs_meta_vector2) { return Variant( *(Vector2*) ptr ); }
	else if (type == GlWorld::glecs_meta_vector2i) { return Variant( *(Vector2i*) ptr ); }
	else if (type == GlWorld::glecs_meta_rect2) { return Variant( *(Rect2*) ptr ); }
	else if (type == GlWorld::glecs_meta_rect2i) { return Variant( *(Rect2i*) ptr ); }
	else if (type == GlWorld::glecs_meta_vector3) { return Variant( *(Vector3*) ptr ); }
	else if (type == GlWorld::glecs_meta_vector3i) { return Variant( *(Vector3i*) ptr ); }
	else if (type == GlWorld::glecs_meta_transform2d) { return Variant( *(Transform2D*) ptr ); }
	else if (type == GlWorld::glecs_meta_vector4) { return Variant( *(Vector4*) ptr ); }
	else if (type == GlWorld::glecs_meta_vector4i) { return Variant( *(Vector4i*) ptr ); }
	else if (type == GlWorld::glecs_meta_plane) { return Variant( *(Plane*) ptr ); }
	else if (type == GlWorld::glecs_meta_quaternion) { return Variant( *(Quaternion*) ptr ); }
	else if (type == GlWorld::glecs_meta_aabb) { return Variant( *(AABB*) ptr ); }
	else if (type == GlWorld::glecs_meta_basis) { return Variant( *(Basis*) ptr ); }
	else if (type == GlWorld::glecs_meta_transform3d) { return Variant( *(Transform3D*) ptr ); }
	else if (type == GlWorld::glecs_meta_projection) { return Variant( *(Projection*) ptr ); }
	else if (type == GlWorld::glecs_meta_color) { return Variant( *(Color*) ptr ); }

	else if (type == GlWorld::glecs_meta_string_name) { return Variant( *(StringName*) ptr ); }
	else if (type == GlWorld::glecs_meta_node_path) { return Variant( *(NodePath*) ptr ); }
	else if (type == GlWorld::glecs_meta_rid) { return Variant( *(RID*) ptr ); }
	else if (type == GlWorld::glecs_meta_object) { return Variant( *(Variant*) ptr ); }
	else if (type == GlWorld::glecs_meta_callable) { return Variant( *(Callable*) ptr ); }
	else if (type == GlWorld::glecs_meta_signal) { return Variant( *(Signal*) ptr ); }
	else if (type == GlWorld::glecs_meta_dictionary) { return *(Variant*) ptr; }
	else if (type == GlWorld::glecs_meta_array) { return *(Variant*) ptr; }
	else if (type == GlWorld::glecs_meta_packed_int32_array) { return Variant( *(PackedInt32Array*) ptr ); }
	else if (type == GlWorld::glecs_meta_packed_int64_array) { return Variant( *(PackedInt64Array*) ptr ); }
	else if (type == GlWorld::glecs_meta_packed_float32_array) { return Variant( *(PackedFloat32Array*) ptr ); }
	else if (type == GlWorld::glecs_meta_packed_float64_array) { return Variant( *(PackedFloat64Array*) ptr ); }
	else if (type == GlWorld::glecs_meta_packed_string_array) { return Variant( *(PackedStringArray*) ptr ); }
	else if (type == GlWorld::glecs_meta_packed_vector2_array) { return Variant( *(PackedVector2Array*) ptr ); }
	else if (type == GlWorld::glecs_meta_packed_vector3_array) { return Variant( *(PackedVector3Array*) ptr ); }
	else if (type == GlWorld::glecs_meta_packed_color_array) { return Variant( *(PackedColorArray*) ptr ); }

	if (ecs_has_id(raw, type, ecs_id(EcsPrimitive))) {
		return member_value_as_primitive(
			ptr,
			ecs_get(raw, type, EcsPrimitive)->kind
		);
	}

	ERR(nullptr,
		"Can't convert type ", ecs_get_name(raw, type), " to Variant"
	);
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
	godot::ClassDB::bind_method(D_METHOD("get_member", "member"), &GlComponent::get_member);
	godot::ClassDB::bind_method(D_METHOD("set_member", "member", "value"), &GlComponent::set_member);

	godot::ClassDB::bind_method(D_METHOD("get_source_entity"), &GlComponent::get_source_entity);
	godot::ClassDB::bind_method(D_METHOD("get_source_id"), &GlComponent::get_source_id);
}
