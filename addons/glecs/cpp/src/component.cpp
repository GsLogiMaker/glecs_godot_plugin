
#include "component.h"
#include "godot_cpp/variant/array.hpp"
#include "godot_cpp/variant/dictionary.hpp"
#include "godot_cpp/variant/variant.hpp"
#include "utils.h"

#include <cstdint>
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
	Variant::Type vari_type = get_world()->id_to_variant_type(type);

	#define SET_MEMBER(variant_type, real_type) \
		EXPECT_VARIANT(value, Variant::variant_type); \
		*(real_type*) ptr = value;

	switch (vari_type) {
	case(Variant::Type::NIL): {
		if (type == GlWorld::glecs_meta_real) {
			SET_MEMBER(FLOAT, real_t); return;
		}
		if (ecs_has_id(raw, type, ecs_id(EcsPrimitive))) {
			return set_member_value_as_primitive(
				ptr,
				value,
				ecs_get(raw, type, EcsPrimitive)->kind
			);
		}

		ERR(/**/,
			"Can't set member\nType ", ecs_get_name(raw, type), " is not handled"
		);
	}
	case(Variant::Type::BOOL): { SET_MEMBER(BOOL, bool); return; }
	case(Variant::Type::INT): { SET_MEMBER(INT, int64_t); return; }
	case(Variant::Type::FLOAT): { SET_MEMBER(FLOAT, float); return; }
	case(Variant::Type::STRING): { SET_MEMBER(STRING, String); return; }
	case(Variant::Type::VECTOR2): { SET_MEMBER(VECTOR2, Vector2); return; }
	case(Variant::Type::VECTOR2I): { SET_MEMBER(VECTOR2I, Vector2i); return; }
	case(Variant::Type::RECT2): { SET_MEMBER(RECT2, Rect2); return; }
	case(Variant::Type::RECT2I): { SET_MEMBER(RECT2I, Rect2i); return; }
	case(Variant::Type::VECTOR3): { SET_MEMBER(VECTOR3, Vector3); return; }
	case(Variant::Type::VECTOR3I): { SET_MEMBER(VECTOR3I, Vector3i); return; }
	case(Variant::Type::TRANSFORM2D): { SET_MEMBER(TRANSFORM2D, Transform2D); return; }
	case(Variant::Type::VECTOR4): { SET_MEMBER(VECTOR4, Vector4); return; }
	case(Variant::Type::VECTOR4I): { SET_MEMBER(VECTOR4I, Vector4i); return; }
	case(Variant::Type::PLANE): { SET_MEMBER(PLANE, Plane); return; }
	case(Variant::Type::QUATERNION): { SET_MEMBER(QUATERNION, Quaternion); return; }
	case(Variant::Type::AABB): { SET_MEMBER(AABB, AABB); return; }
	case(Variant::Type::BASIS): { SET_MEMBER(BASIS, Basis); return; }
	case(Variant::Type::TRANSFORM3D): { SET_MEMBER(TRANSFORM3D, Transform3D); return; }
	case(Variant::Type::PROJECTION): { SET_MEMBER(PROJECTION, Projection); return; }
	case(Variant::Type::COLOR): { SET_MEMBER(COLOR, Color); return; }
	case(Variant::Type::STRING_NAME): { SET_MEMBER(STRING_NAME, StringName); return; }
	case(Variant::Type::NODE_PATH): { SET_MEMBER(NODE_PATH, NodePath); return; }
	case(Variant::Type::RID): { SET_MEMBER(RID, RID); return; }
	case(Variant::Type::OBJECT): { SET_MEMBER(OBJECT, Variant); return; }
	case(Variant::Type::CALLABLE): { SET_MEMBER(CALLABLE, Callable); return; }
	case(Variant::Type::SIGNAL): { SET_MEMBER(SIGNAL, Signal); return; }
	case(Variant::Type::DICTIONARY): { SET_MEMBER(DICTIONARY, Dictionary); return; }
	case(Variant::Type::ARRAY): { SET_MEMBER(ARRAY, Array); return; }
	case(Variant::Type::PACKED_BYTE_ARRAY): { SET_MEMBER(PACKED_BYTE_ARRAY, PackedByteArray); return; }
	case(Variant::Type::PACKED_INT32_ARRAY): { SET_MEMBER(PACKED_INT32_ARRAY, PackedInt32Array); return; }
	case(Variant::Type::PACKED_INT64_ARRAY): { SET_MEMBER(PACKED_INT64_ARRAY, PackedInt64Array); return; }
	case(Variant::Type::PACKED_FLOAT32_ARRAY): { SET_MEMBER(PACKED_FLOAT32_ARRAY, PackedFloat32Array); return; }
	case(Variant::Type::PACKED_FLOAT64_ARRAY): { SET_MEMBER(PACKED_FLOAT64_ARRAY, PackedFloat64Array); return; }
	case(Variant::Type::PACKED_STRING_ARRAY): { SET_MEMBER(PACKED_STRING_ARRAY, PackedStringArray); return; }
	case(Variant::Type::PACKED_VECTOR2_ARRAY): { SET_MEMBER(PACKED_VECTOR2_ARRAY, PackedVector2Array); return; }
	case(Variant::Type::PACKED_VECTOR3_ARRAY): { SET_MEMBER(PACKED_VECTOR3_ARRAY, PackedVector3Array); return; }
	case(Variant::Type::PACKED_COLOR_ARRAY): { SET_MEMBER(PACKED_COLOR_ARRAY, PackedColorArray); return; }
	case(Variant::Type::VARIANT_MAX): throw "Can't set set member\\nVARIANt_MAX is not a valid type";
	}

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
		case ecs_primitive_kind_t::EcsUPtr: ERR(nullptr, "Can't get primitive\nCan't hanlde uptr");
		case ecs_primitive_kind_t::EcsIPtr: ERR(nullptr, "Can't get primitive\nCan't hanlde iptr");
		case ecs_primitive_kind_t::EcsString: return *(char**) ptr;
		case ecs_primitive_kind_t::EcsEntity: return *(ecs_entity_t*) ptr;
		case ecs_primitive_kind_t::EcsId: return *(ecs_entity_t*) ptr;
		default: ERR(nullptr, "Can't get primitive\nUnknown primitive type");
	}
}

Variant GlComponent::member_value_as_type(
	void* ptr,
	ecs_entity_t type
) {
	ecs_world_t* raw = get_world()->raw();
	Variant::Type vari_type = get_world()->id_to_variant_type(type);

	switch (vari_type) {
	case(Variant::Type::NIL): {
		// Member is not a Godot type. Try to get from Flecs primitive
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
	case(Variant::Type::BOOL): return Variant( *(bool*) ptr );
	case(Variant::Type::INT): return Variant( *(int64_t*) ptr );
	case(Variant::Type::FLOAT): return Variant( *(float*) ptr );
	case(Variant::Type::STRING): return Variant( *(String*) ptr );
	case(Variant::Type::VECTOR2): return Variant( *(Vector2*) ptr );
	case(Variant::Type::VECTOR2I): return Variant( *(Vector2i*) ptr );
	case(Variant::Type::RECT2): return Variant( *(Rect2*) ptr );
	case(Variant::Type::RECT2I): return Variant( *(Rect2i*) ptr );
	case(Variant::Type::VECTOR3): return Variant( *(Vector3*) ptr );
	case(Variant::Type::VECTOR3I): return Variant( *(Vector3i*) ptr );
	case(Variant::Type::TRANSFORM2D): return Variant( *(Transform2D*) ptr );
	case(Variant::Type::VECTOR4): return Variant( *(Vector4*) ptr );
	case(Variant::Type::VECTOR4I): return Variant( *(Vector4i*) ptr );
	case(Variant::Type::PLANE): return Variant( *(Plane*) ptr );
	case(Variant::Type::QUATERNION): return Variant( *(Quaternion*) ptr );
	case(Variant::Type::AABB): return Variant( *(AABB*) ptr );
	case(Variant::Type::BASIS): return Variant( *(Basis*) ptr );
	case(Variant::Type::TRANSFORM3D): return Variant( *(Transform3D*) ptr );
	case(Variant::Type::PROJECTION): return Variant( *(Projection*) ptr );
	case(Variant::Type::COLOR): return Variant( *(Color*) ptr );
	case(Variant::Type::STRING_NAME): return Variant( *(StringName*) ptr );
	case(Variant::Type::NODE_PATH): return Variant( *(NodePath*) ptr );
	case(Variant::Type::RID): return Variant( *(RID*) ptr );
	case(Variant::Type::OBJECT): return Variant( *(Variant*) ptr );
	case(Variant::Type::CALLABLE): return Variant( *(Callable*) ptr );
	case(Variant::Type::SIGNAL): return Variant( *(Signal*) ptr );
	case(Variant::Type::DICTIONARY): return *(Dictionary*) ptr;
	case(Variant::Type::ARRAY): return *(Array*) ptr;
	case(Variant::Type::PACKED_BYTE_ARRAY): return Variant( *(PackedByteArray*) ptr );
	case(Variant::Type::PACKED_INT32_ARRAY): return Variant( *(PackedInt32Array*) ptr );
	case(Variant::Type::PACKED_INT64_ARRAY): return Variant( *(PackedInt64Array*) ptr );
	case(Variant::Type::PACKED_FLOAT32_ARRAY): return Variant( *(PackedFloat32Array*) ptr );
	case(Variant::Type::PACKED_FLOAT64_ARRAY): return Variant( *(PackedFloat64Array*) ptr );
	case(Variant::Type::PACKED_STRING_ARRAY): return Variant( *(PackedStringArray*) ptr );
	case(Variant::Type::PACKED_VECTOR2_ARRAY): return Variant( *(PackedVector2Array*) ptr );
	case(Variant::Type::PACKED_VECTOR3_ARRAY): return Variant( *(PackedVector3Array*) ptr );
	case(Variant::Type::PACKED_COLOR_ARRAY): return Variant( *(PackedColorArray*) ptr );
	case(Variant::Type::VARIANT_MAX): throw "Can't get type VARIANT_MAX";
	}

	throw "Unreachable";
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
