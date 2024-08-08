
#include "world.h"
#include "entity.h"
#include "component_builder.h"
#include "godot_cpp/core/memory.hpp"
#include "godot_cpp/variant/dictionary.hpp"
#include "godot_cpp/variant/utility_functions.hpp"
#include "godot_cpp/variant/variant.hpp"
#include "utils.h"

#include <flecs.h>
#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/classes/engine.hpp>
#include <godot_cpp/variant/string_name.hpp>

using namespace godot;

ecs_entity_t GlWorld::glecs = 0;
ecs_entity_t GlWorld::glecs_meta = 0;
ecs_entity_t GlWorld::glecs_meta_real = 0;
ecs_entity_t GlWorld::glecs_meta_nil = 0;
ecs_entity_t GlWorld::glecs_meta_bool = 0;
ecs_entity_t GlWorld::glecs_meta_int = 0;
ecs_entity_t GlWorld::glecs_meta_float = 0;
ecs_entity_t GlWorld::glecs_meta_string = 0;
ecs_entity_t GlWorld::glecs_meta_vector2 = 0;
ecs_entity_t GlWorld::glecs_meta_vector2i = 0;
ecs_entity_t GlWorld::glecs_meta_rect2 = 0;
ecs_entity_t GlWorld::glecs_meta_rect2i = 0;
ecs_entity_t GlWorld::glecs_meta_vector3 = 0;
ecs_entity_t GlWorld::glecs_meta_vector3i = 0;
ecs_entity_t GlWorld::glecs_meta_transform2d = 0;
ecs_entity_t GlWorld::glecs_meta_vector4 = 0;
ecs_entity_t GlWorld::glecs_meta_vector4i = 0;
ecs_entity_t GlWorld::glecs_meta_plane = 0;
ecs_entity_t GlWorld::glecs_meta_quaternion = 0;
ecs_entity_t GlWorld::glecs_meta_aabb = 0;
ecs_entity_t GlWorld::glecs_meta_basis = 0;
ecs_entity_t GlWorld::glecs_meta_transform3d = 0;
ecs_entity_t GlWorld::glecs_meta_projection = 0;
ecs_entity_t GlWorld::glecs_meta_color = 0;
ecs_entity_t GlWorld::glecs_meta_string_name = 0;
ecs_entity_t GlWorld::glecs_meta_node_path = 0;
ecs_entity_t GlWorld::glecs_meta_rid = 0;
ecs_entity_t GlWorld::glecs_meta_object = 0;
ecs_entity_t GlWorld::glecs_meta_callable = 0;
ecs_entity_t GlWorld::glecs_meta_signal = 0;
ecs_entity_t GlWorld::glecs_meta_dictionary = 0;
ecs_entity_t GlWorld::glecs_meta_array = 0;
ecs_entity_t GlWorld::glecs_meta_packed_byte_array = 0;
ecs_entity_t GlWorld::glecs_meta_packed_int32_array = 0;
ecs_entity_t GlWorld::glecs_meta_packed_int64_array = 0;
ecs_entity_t GlWorld::glecs_meta_packed_float32_array = 0;
ecs_entity_t GlWorld::glecs_meta_packed_float64_array = 0;
ecs_entity_t GlWorld::glecs_meta_packed_string_array = 0;
ecs_entity_t GlWorld::glecs_meta_packed_vector2_array = 0;
ecs_entity_t GlWorld::glecs_meta_packed_vector3_array = 0;
ecs_entity_t GlWorld::glecs_meta_packed_color_array = 0;

GlWorld::GlWorld() {
	_raw = ecs_init();
	ECS_IMPORT(raw(), FlecsStats);

	// Add glecs module
	ecs_component_desc_t empty_desc = {0};
	glecs = ecs_module_init(
		_raw,
		"glecs",
		&empty_desc
	);

	// Add glecs/meta module
	{
		ecs_entity_t old_scope = ecs_get_scope(_raw);
		ecs_set_scope(_raw, glecs);

		ecs_component_desc_t comp_desc = {
			.entity = ecs_new_from_path(_raw, glecs, "meta"),
		};
		glecs_meta = ecs_module_init(
			_raw,
			"meta",
			&comp_desc
		);

		ecs_set_scope(_raw, old_scope);
	}

	{ // Add glecs/meta/Real type
		ecs_primitive_kind_t kind;
		if (sizeof(real_t) == sizeof(float)) {
			kind = ecs_primitive_kind_t::EcsF32;
		} else if (sizeof(real_t) == sizeof(double)) {
			kind = ecs_primitive_kind_t::EcsF64;
		} else {
			throw "Godot's real_t type is somehow not a float or double";
		}
		ecs_primitive_desc_t primi_desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Real"),
			.kind = kind
		};
		glecs_meta_real = ecs_primitive_init(_raw, &primi_desc);
	}

	glecs_meta_nil = ecs_new(_raw);
	glecs_meta_bool = ecs_new(_raw);
	glecs_meta_int = ecs_new(_raw);
	glecs_meta_float = ecs_new(_raw);
	glecs_meta_string = ecs_new(_raw);
	glecs_meta_vector2 = ecs_new(_raw);
	glecs_meta_vector2i = ecs_new(_raw);
	glecs_meta_rect2 = ecs_new(_raw);
	glecs_meta_rect2i = ecs_new(_raw);
	glecs_meta_vector3 = ecs_new(_raw);
	glecs_meta_vector3i = ecs_new(_raw);
	glecs_meta_transform2d = ecs_new(_raw);
	glecs_meta_vector4 = ecs_new(_raw);
	glecs_meta_vector4i = ecs_new(_raw);
	glecs_meta_plane = ecs_new(_raw);
	glecs_meta_quaternion = ecs_new(_raw);
	glecs_meta_aabb = ecs_new(_raw);
	glecs_meta_basis = ecs_new(_raw);
	glecs_meta_transform3d = ecs_new(_raw);
	glecs_meta_projection = ecs_new(_raw);
	glecs_meta_color = ecs_new(_raw);
	glecs_meta_string_name = ecs_new(_raw);
	glecs_meta_node_path = ecs_new(_raw);
	glecs_meta_rid = ecs_new(_raw);
	glecs_meta_object = ecs_new(_raw);
	glecs_meta_callable = ecs_new(_raw);
	glecs_meta_signal = ecs_new(_raw);
	glecs_meta_dictionary = ecs_new(_raw);
	glecs_meta_array = ecs_new(_raw);
	glecs_meta_packed_byte_array = ecs_new(_raw);
	glecs_meta_packed_int32_array = ecs_new(_raw);
	glecs_meta_packed_int64_array = ecs_new(_raw);
	glecs_meta_packed_float32_array = ecs_new(_raw);
	glecs_meta_packed_float64_array = ecs_new(_raw);
	glecs_meta_packed_string_array = ecs_new(_raw);
	glecs_meta_packed_vector2_array = ecs_new(_raw);
	glecs_meta_packed_vector3_array = ecs_new(_raw);
	glecs_meta_packed_color_array = ecs_new(_raw);

	define_gd_literal("nil", ecs_primitive_kind_t::EcsUPtr, &glecs_meta_nil);
	define_gd_literal("bool", ecs_primitive_kind_t::EcsBool, &glecs_meta_bool);
	define_gd_literal("int", ecs_primitive_kind_t::EcsI64, &glecs_meta_int);
	define_gd_literal("float", ecs_primitive_kind_t::EcsF64, &glecs_meta_float);
	define_gd_component<String>("String", &glecs_meta_string);

	{ // Add glecs/meta/Vector2 type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_vector2,
			.members = {
				{.name = "x", .type = glecs_meta_real},
				{.name = "y", .type = glecs_meta_real}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_vector2,
			glecs_meta,
			"Vector2",
			"/",
			"/root/"
		);
	}

	{ // Add glecs/meta/Vector2i type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_vector2i,
			.members = {
				{.name = "x", .type = ecs_id(ecs_i32_t)},
				{.name = "y", .type = ecs_id(ecs_i32_t)}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_vector2i,
			glecs_meta,
			"Vector2I",
			"/",
			"/root/"
		);
	}

	{ // Add glecs/meta/Rect type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_rect2,
			.members = {
				{.name = "position", .type = glecs_meta_vector2},
				{.name = "size", .type = glecs_meta_vector2}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_rect2,
			glecs_meta,
			"Rect2",
			"/",
			"/root/"
		);
	}

	{ // Add glecs/meta/Rect2i type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_rect2i,
			.members = {
				{.name = "position", .type = glecs_meta_vector2i},
				{.name = "size", .type = glecs_meta_vector2i}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_rect2i,
			glecs_meta,
			"Rect2i",
			"/",
			"/root/"
		);
	}

	{ // Add glecs/meta/Vector3 type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_vector3,
			.members = {
				{.name = "x", .type = glecs_meta_real},
				{.name = "y", .type = glecs_meta_real},
				{.name = "z", .type = glecs_meta_real}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_vector3,
			glecs_meta,
			"Vector3",
			"/",
			"/root/"
		);
	}

	{ // Add glecs/meta/Vector3i type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_vector3i,
			.members = {
				{.name = "x", .type = ecs_id(ecs_i32_t)},
				{.name = "y", .type = ecs_id(ecs_i32_t)},
				{.name = "z", .type = ecs_id(ecs_i32_t)}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_vector3i,
			glecs_meta,
			"Vector3i",
			"/",
			"/root/"
		);
	}

	{ // Add glecs/meta/Transform2D type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_transform2d,
			.members = {
				{.name = "x", .type = glecs_meta_vector2},
				{.name = "y", .type = glecs_meta_vector2},
				{.name = "origin", .type = glecs_meta_vector2}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_transform2d,
			glecs_meta,
			"Transform2D",
			"/",
			"/root/"
		);
	}

	{ // Add glecs/meta/Vector4 type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_vector4,
			.members = {
				{.name = "x", .type = glecs_meta_real},
				{.name = "y", .type = glecs_meta_real},
				{.name = "z", .type = glecs_meta_real},
				{.name = "w", .type = glecs_meta_real}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_vector4,
			glecs_meta,
			"Vector4",
			"/",
			"/root/"
		);
	}

	{ // Add glecs/meta/Vector4i type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_vector4i,
			.members = {
				{.name = "x", .type = ecs_id(ecs_i32_t)},
				{.name = "y", .type = ecs_id(ecs_i32_t)},
				{.name = "z", .type = ecs_id(ecs_i32_t)},
				{.name = "w", .type = ecs_id(ecs_i32_t)}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_vector4i,
			glecs_meta,
			"Vector4i",
			"/",
			"/root/"
		);
	}

	{ // Add glecs/meta/Plane type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_plane,
			.members = {
				{.name = "x", .type = glecs_meta_real},
				{.name = "y", .type = glecs_meta_real},
				{.name = "z", .type = glecs_meta_real},
				{.name = "d", .type = glecs_meta_real},
				{.name = "normal", .type = glecs_meta_vector3}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_plane,
			glecs_meta,
			"Plane",
			"/",
			"/root/"
		);
	}

	{ // Add glecs/meta/Quaternion type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_quaternion,
			.members = {
				{.name = "x", .type = glecs_meta_real},
				{.name = "y", .type = glecs_meta_real},
				{.name = "z", .type = glecs_meta_real},
				{.name = "w", .type = glecs_meta_real}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_quaternion,
			glecs_meta,
			"Quaternion",
			"/",
			"/root/"
		);
	}

	{ // Add glecs/meta/AABB type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_aabb,
			.members = {
				{.name = "position", .type = glecs_meta_vector3},
				{.name = "size", .type = glecs_meta_vector3}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_aabb,
			glecs_meta,
			"AABB",
			"/",
			"/root/"
		);
	}

	{ // Add glecs/meta/Basis type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_basis,
			.members = {
				{.name = "x", .type = glecs_meta_vector3},
				{.name = "y", .type = glecs_meta_vector3},
				{.name = "z", .type = glecs_meta_vector3}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_basis,
			glecs_meta,
			"Basis",
			"/",
			"/root/"
		);
	}

	{ // Add glecs/meta/Transform3D type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_transform3d,
			.members = {
				{.name = "basis", .type = glecs_meta_basis},
				{.name = "origin", .type = glecs_meta_vector3}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_transform3d,
			glecs_meta,
			"Transform3D",
			"/",
			"/root/"
		);
	}

	{ // Add glecs/meta/Projection type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_projection,
			.members = {
				{.name = "x", .type = glecs_meta_vector4},
				{.name = "y", .type = glecs_meta_vector4},
				{.name = "z", .type = glecs_meta_vector4},
				{.name = "w", .type = glecs_meta_vector4}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_projection,
			glecs_meta,
			"Projection",
			"/",
			"/root/"
		);
	}

	{ // Add glecs/meta/Color type
		ecs_struct_desc_t desc = {
			.entity = glecs_meta_color,
			.members = {
				{.name = "r", .type = ecs_id(ecs_f32_t)},
				{.name = "g", .type = ecs_id(ecs_f32_t)},
				{.name = "b", .type = ecs_id(ecs_f32_t)},
				{.name = "a", .type = ecs_id(ecs_f32_t)}
			}
		}; ecs_struct_init(_raw, &desc);
		ecs_add_path_w_sep(
			_raw,
			glecs_meta_color,
			glecs_meta,
			"Color",
			"/",
			"/root/"
		);
	}

	define_gd_component<StringName>("StringName", &glecs_meta_string_name);
	define_gd_component<NodePath>("NodePath", &glecs_meta_node_path);
	define_gd_component<RID>("RID", &glecs_meta_rid);
	define_gd_component<Variant>("Object", &glecs_meta_object);
	define_gd_component<Callable>("Callable", &glecs_meta_callable);
	define_gd_component<Signal>("Signal", &glecs_meta_signal);
	define_gd_component<Dictionary>("Dictionary", &glecs_meta_dictionary);
	define_gd_component<Array>("Array", &glecs_meta_array);
	define_gd_component<PackedByteArray>("PackedByteArray", &glecs_meta_packed_byte_array);
	define_gd_component<PackedInt32Array>("PackedInt32Array", &glecs_meta_packed_int32_array);
	define_gd_component<PackedInt64Array>("PackedInt64Array", &glecs_meta_packed_int64_array);
	define_gd_component<PackedFloat32Array>("PackedFloat32Array", &glecs_meta_packed_float32_array);
	define_gd_component<PackedFloat64Array>("PackedFloat64Array", &glecs_meta_packed_float64_array);
	define_gd_component<PackedStringArray>("PackedStringArray", &glecs_meta_packed_string_array);
	define_gd_component<PackedVector2Array>("PackedVector2Array", &glecs_meta_packed_vector2_array);
	define_gd_component<PackedVector3Array>("PackedVector3Array", &glecs_meta_packed_vector3_array);
	define_gd_component<PackedColorArray>("PackedColorArray", &glecs_meta_packed_color_array);

	#undef DEFINE_GD_COMPONENT
	#undef DEFINE_GD_COMPONENT_WITH_HOOKS
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

ecs_entity_t GlWorld::variant_type_to_id(Variant::Type type) {
	if (type == Variant::Type::VARIANT_MAX) {
		throw "No ID exists for VARIANT_MAX";
	}
	return GlWorld::glecs_meta_nil + type;
}

Variant::Type GlWorld::id_to_variant_type(ecs_entity_t id) {
	if (id < GlWorld::glecs_meta_nil) {
		return godot::Variant::NIL;
	}
	Variant::Type type = Variant::Type(id - GlWorld::glecs_meta_nil);
	if (type >= Variant::Type::VARIANT_MAX) {
		return godot::Variant::NIL;
	}
	return type;
}

// ----------------------------------------------
// --- Unexposed ---
// ----------------------------------------------

void GlWorld::copy_component_ptr(
	const void* src_ptr,
	void* dst_ptr,
	ecs_entity_t component
) {
	const EcsStruct* c_struct = ecs_get(_raw, component, EcsStruct);
	if (c_struct == nullptr) {
		return;
	}
	for (int i=0; i != ecs_vec_count(&c_struct->members); i++) {
		const ecs_member_t* member = ecs_vec_get_t(&c_struct->members, ecs_member_t, i);
		auto src_member_value = (void*) ((int8_t*)src_ptr + member->offset);
		auto dst_member_value = (void*) ((int8_t*)dst_ptr + member->offset);
		copy_gd_type_ptr(src_member_value, dst_member_value, member->type);
	}
}

void GlWorld::copy_gd_type_ptr(
	const void* src_ptr,
	void* dst_ptr,
	ecs_entity_t type
) {
	Variant::Type vari_type = id_to_variant_type(type);

	switch (vari_type) {
	case(Variant::Type::NIL): {
		if (ecs_has(_raw, type, EcsStruct)) {
			copy_component_ptr(src_ptr, dst_ptr, type);
		};
		break;
	}
	case(Variant::Type::BOOL): *(bool*)dst_ptr = *(bool*)src_ptr; break;
	case(Variant::Type::INT): *(int64_t*)dst_ptr = *(int64_t*)src_ptr; break;
	case(Variant::Type::FLOAT): *(float*)dst_ptr = *(float*)src_ptr; break;
	case(Variant::Type::STRING): *(String*)dst_ptr = *(String*)src_ptr; break;
	case(Variant::Type::VECTOR2): *(Vector2*)dst_ptr = *(Vector2*)src_ptr; break;
	case(Variant::Type::VECTOR2I): *(Vector2i*)dst_ptr = *(Vector2i*)src_ptr; break;
	case(Variant::Type::RECT2): *(Rect2*)dst_ptr = *(Rect2*)src_ptr; break;
	case(Variant::Type::RECT2I): *(Rect2i*)dst_ptr = *(Rect2i*)src_ptr; break;
	case(Variant::Type::VECTOR3): *(Vector3*)dst_ptr = *(Vector3*)src_ptr; break;
	case(Variant::Type::VECTOR3I): *(Vector3i*)dst_ptr = *(Vector3i*)src_ptr; break;
	case(Variant::Type::TRANSFORM2D): *(Transform2D*)dst_ptr = *(Transform2D*)src_ptr; break;
	case(Variant::Type::VECTOR4): *(Vector4*)dst_ptr = *(Vector4*)src_ptr; break;
	case(Variant::Type::VECTOR4I): *(Vector4i*)dst_ptr = *(Vector4i*)src_ptr; break;
	case(Variant::Type::PLANE): *(Plane*)dst_ptr = *(Plane*)src_ptr; break;
	case(Variant::Type::QUATERNION): *(Quaternion*)dst_ptr = *(Quaternion*)src_ptr; break;
	case(Variant::Type::AABB): *(AABB*)dst_ptr = *(AABB*)src_ptr; break;
	case(Variant::Type::BASIS): *(Basis*)dst_ptr = *(Basis*)src_ptr; break;
	case(Variant::Type::TRANSFORM3D): *(Transform3D*)dst_ptr = *(Transform3D*)src_ptr; break;
	case(Variant::Type::PROJECTION): *(Projection*)dst_ptr = *(Projection*)src_ptr; break;
	case(Variant::Type::COLOR): *(Color*)dst_ptr = *(Color*)src_ptr; break;
	case(Variant::Type::STRING_NAME): *(StringName*)dst_ptr = *(StringName*)src_ptr; break;
	case(Variant::Type::NODE_PATH): *(NodePath*)dst_ptr = *(NodePath*)src_ptr; break;
	case(Variant::Type::RID): *(RID*)dst_ptr = *(RID*)src_ptr; break;
	case(Variant::Type::OBJECT): *(Variant*)dst_ptr = *(Variant*)src_ptr; break;
	case(Variant::Type::CALLABLE): *(Callable*)dst_ptr = *(Callable*)src_ptr; break;
	case(Variant::Type::SIGNAL): *(Signal*)dst_ptr = *(Signal*)src_ptr; break;
	case(Variant::Type::DICTIONARY): *(Dictionary*)dst_ptr = *(Dictionary*)src_ptr; break;
	case(Variant::Type::ARRAY): *(Array*)dst_ptr = *(Array*)src_ptr; break;
	case(Variant::Type::PACKED_BYTE_ARRAY): *(PackedByteArray*)dst_ptr = *(PackedByteArray*)src_ptr; break;
	case(Variant::Type::PACKED_INT32_ARRAY): *(PackedInt32Array*)dst_ptr = *(PackedInt32Array*)src_ptr; break;
	case(Variant::Type::PACKED_INT64_ARRAY): *(PackedInt64Array*)dst_ptr = *(PackedInt64Array*)src_ptr; break;
	case(Variant::Type::PACKED_FLOAT32_ARRAY): *(PackedFloat32Array*)dst_ptr = *(PackedFloat32Array*)src_ptr; break;
	case(Variant::Type::PACKED_FLOAT64_ARRAY): *(PackedFloat64Array*)dst_ptr = *(PackedFloat64Array*)src_ptr; break;
	case(Variant::Type::PACKED_STRING_ARRAY): *(PackedStringArray*)dst_ptr = *(PackedStringArray*)src_ptr; break;
	case(Variant::Type::PACKED_VECTOR2_ARRAY): *(PackedVector2Array*)dst_ptr = *(PackedVector2Array*)src_ptr; break;
	case(Variant::Type::PACKED_VECTOR3_ARRAY): *(PackedVector3Array*)dst_ptr = *(PackedVector3Array*)src_ptr; break;
	case(Variant::Type::PACKED_COLOR_ARRAY): *(PackedColorArray*)dst_ptr = *(PackedColorArray*)src_ptr; break;
	case(Variant::Type::VARIANT_MAX): throw "VARIANT_MAX can't be deinitialized";
	}
}

void GlWorld::deinit_component_ptr(
	void* ptr,
	ecs_entity_t component
) {
	const EcsStruct* c_struct = ecs_get(_raw, component, EcsStruct);
	if (c_struct == nullptr) {
		return;
	}
	for (int i=0; i != ecs_vec_count(&c_struct->members); i++) {
		const ecs_member_t* member = ecs_vec_get_t(&c_struct->members, ecs_member_t, i);
		auto member_value = (void*) ((int8_t*)ptr + member->offset);
		deinit_gd_type_ptr(member_value, member->type);
	}
}

void GlWorld::deinit_gd_type_ptr(
	void* ptr,
	ecs_entity_t type
) {
	Variant::Type vari_type = id_to_variant_type(type);

	switch (vari_type) {
	case(Variant::Type::NIL): {
		if (ecs_has(_raw, type, EcsStruct)) {
			deinit_component_ptr(ptr, type);
		};
		break;
	}
	case(Variant::Type::BOOL): break;
	case(Variant::Type::INT): break;
	case(Variant::Type::FLOAT): break;
	case(Variant::Type::STRING): (*(String*)ptr).~String(); break;
	case(Variant::Type::VECTOR2): (*(Vector2*)ptr).~Vector2(); break;
	case(Variant::Type::VECTOR2I): (*(Vector2i*)ptr).~Vector2i(); break;
	case(Variant::Type::RECT2): (*(Rect2*)ptr).~Rect2(); break;
	case(Variant::Type::RECT2I): (*(Rect2i*)ptr).~Rect2i(); break;
	case(Variant::Type::VECTOR3): (*(Vector3*)ptr).~Vector3(); break;
	case(Variant::Type::VECTOR3I): (*(Vector3i*)ptr).~Vector3i(); break;
	case(Variant::Type::TRANSFORM2D): (*(Transform2D*)ptr).~Transform2D(); break;
	case(Variant::Type::VECTOR4): (*(Vector4*)ptr).~Vector4(); break;
	case(Variant::Type::VECTOR4I): (*(Vector4i*)ptr).~Vector4i(); break;
	case(Variant::Type::PLANE): (*(Plane*)ptr).~Plane(); break;
	case(Variant::Type::QUATERNION): (*(Quaternion*)ptr).~Quaternion(); break;
	case(Variant::Type::AABB): (*(AABB*)ptr).~AABB(); break;
	case(Variant::Type::BASIS): (*(Basis*)ptr).~Basis(); break;
	case(Variant::Type::TRANSFORM3D): (*(Transform3D*)ptr).~Transform3D(); break;
	case(Variant::Type::PROJECTION): (*(Projection*)ptr).~Projection(); break;
	case(Variant::Type::COLOR): (*(Color*)ptr).~Color(); break;
	case(Variant::Type::STRING_NAME): (*(StringName*)ptr).~StringName(); break;
	case(Variant::Type::NODE_PATH): (*(NodePath*)ptr).~NodePath(); break;
	case(Variant::Type::RID): (*(RID*)ptr).~RID(); break;
	case(Variant::Type::OBJECT): (*(Variant*)ptr).~Variant() ; break;
	case(Variant::Type::CALLABLE): (*(Callable*)ptr).~Callable(); break;
	case(Variant::Type::SIGNAL): (*(Signal*)ptr).~Signal(); break;
	case(Variant::Type::DICTIONARY): (*(Dictionary*)ptr).~Dictionary(); break;
	case(Variant::Type::ARRAY): (*(Array*)ptr).~Array(); break;
	case(Variant::Type::PACKED_BYTE_ARRAY): (*(PackedByteArray*)ptr).~PackedByteArray(); break;
	case(Variant::Type::PACKED_INT32_ARRAY): (*(PackedInt32Array*)ptr).~PackedInt32Array(); break;
	case(Variant::Type::PACKED_INT64_ARRAY): (*(PackedInt64Array*)ptr).~PackedInt64Array(); break;
	case(Variant::Type::PACKED_FLOAT32_ARRAY): (*(PackedFloat32Array*)ptr).~PackedFloat32Array(); break;
	case(Variant::Type::PACKED_FLOAT64_ARRAY): (*(PackedFloat64Array*)ptr).~PackedFloat64Array(); break;
	case(Variant::Type::PACKED_STRING_ARRAY): (*(PackedStringArray*)ptr).~PackedStringArray(); break;
	case(Variant::Type::PACKED_VECTOR2_ARRAY): (*(PackedVector2Array*)ptr).~PackedVector2Array(); break;
	case(Variant::Type::PACKED_VECTOR3_ARRAY): (*(PackedVector3Array*)ptr).~PackedVector3Array(); break;
	case(Variant::Type::PACKED_COLOR_ARRAY): (*(PackedColorArray*)ptr).~PackedColorArray(); break;
	case(Variant::Type::VARIANT_MAX): throw "VARIANT_MAX can't be deinitialized";
	}
}

void GlWorld::init_component_ptr(
	void* ptr,
	ecs_entity_t component,
	Variant args
) {
	const EcsStruct* c_struct = ecs_get(_raw, component, EcsStruct);
	if (c_struct == nullptr) {
		return;
	}
	for (int i=0; i != ecs_vec_count(&c_struct->members); i++) {
		const ecs_member_t* member = ecs_vec_get_t(&c_struct->members, ecs_member_t, i);
		auto member_value = (void*) ((int8_t*)ptr + member->offset);
		init_gd_type_ptr(member_value, member->type);
	}
}

/// If the type is a Variant, then initializes the pointer as that type
void GlWorld::init_gd_type_ptr(
	void* ptr,
	ecs_entity_t type
) {
	Variant::Type vari_type = id_to_variant_type(type);

	switch (vari_type) {
	case(Variant::Type::NIL): if (ecs_has(_raw, type, EcsStruct)) {init_component_ptr(ptr, type, Variant());}; break;
	case(Variant::Type::BOOL): *(bool*) ptr = false; break;
	case(Variant::Type::INT): *(int*) ptr = 0; break;
	case(Variant::Type::FLOAT): *(float*) ptr = 0; break;
	case(Variant::Type::STRING): new(ptr) String(); break;
	case(Variant::Type::VECTOR2): new(ptr) Vector2(); break;
	case(Variant::Type::VECTOR2I): new(ptr) Vector2i(); break;
	case(Variant::Type::RECT2): new(ptr) Rect2(); break;
	case(Variant::Type::RECT2I): new(ptr) Rect2i(); break;
	case(Variant::Type::VECTOR3): new(ptr) Vector3(); break;
	case(Variant::Type::VECTOR3I): new(ptr) Vector3i(); break;
	case(Variant::Type::TRANSFORM2D): new(ptr) Transform2D(); break;
	case(Variant::Type::VECTOR4): new(ptr) Vector4(); break;
	case(Variant::Type::VECTOR4I): new(ptr) Vector4i(); break;
	case(Variant::Type::PLANE): new(ptr) Plane(); break;
	case(Variant::Type::QUATERNION): new(ptr) Quaternion(); break;
	case(Variant::Type::AABB): new(ptr) AABB(); break;
	case(Variant::Type::BASIS): new(ptr) Basis(); break;
	case(Variant::Type::TRANSFORM3D): new(ptr) Transform3D(); break;
	case(Variant::Type::PROJECTION): new(ptr) Projection(); break;
	case(Variant::Type::COLOR): new(ptr) Color(); break;
	case(Variant::Type::STRING_NAME): new(ptr) StringName(); break;
	case(Variant::Type::NODE_PATH): new(ptr) NodePath(); break;
	case(Variant::Type::RID): new(ptr) RID(); break;
	case(Variant::Type::OBJECT): new(ptr) Variant(nullptr) ; break;
	case(Variant::Type::CALLABLE): new(ptr) Callable(); break;
	case(Variant::Type::SIGNAL): new(ptr) Signal(); break;
	case(Variant::Type::DICTIONARY): new(ptr) Dictionary(); break;
	case(Variant::Type::ARRAY): new(ptr) Array(); break;
	case(Variant::Type::PACKED_BYTE_ARRAY): new(ptr) PackedByteArray(); break;
	case(Variant::Type::PACKED_INT32_ARRAY): new(ptr) PackedInt32Array(); break;
	case(Variant::Type::PACKED_INT64_ARRAY): new(ptr) PackedInt64Array(); break;
	case(Variant::Type::PACKED_FLOAT32_ARRAY): new(ptr) PackedFloat32Array(); break;
	case(Variant::Type::PACKED_FLOAT64_ARRAY): new(ptr) PackedFloat64Array(); break;
	case(Variant::Type::PACKED_STRING_ARRAY): new(ptr) PackedStringArray(); break;
	case(Variant::Type::PACKED_VECTOR2_ARRAY): new(ptr) PackedVector2Array(); break;
	case(Variant::Type::PACKED_VECTOR3_ARRAY): new(ptr) PackedVector3Array(); break;
	case(Variant::Type::PACKED_COLOR_ARRAY): new(ptr) PackedColorArray(); break;
	case(Variant::Type::VARIANT_MAX): throw "VARIANT_MAX can't be initialized";
	}
}

ecs_world_t * GlWorld::raw() {
	return _raw;
}

// ----------------------------------------------
// --- Protected ---
// ----------------------------------------------

void GlWorld::_bind_methods() {
	godot::ClassDB::bind_method(D_METHOD("component_builder"), &GlWorld::component_builder);
	godot::ClassDB::bind_method(D_METHOD("coerce_id", "entity"), &GlWorld::coerce_id);
	godot::ClassDB::bind_method(D_METHOD("start_rest_api"), &GlWorld::start_rest_api);
	godot::ClassDB::bind_method(D_METHOD("progress", "delta"), &GlWorld::progress);
}

// ----------------------------------------------
// --- Private ---
// ----------------------------------------------

void GlWorld::define_gd_literal(
	const char* name,
	ecs_primitive_kind_t primitive,
	ecs_entity_t* id_storage
) {
	ecs_primitive_desc_t desc = {
		.entity = *id_storage,
		.kind = primitive
	}; ecs_primitive_init(_raw, &desc);
	ecs_add_path_w_sep(
		_raw,
		*id_storage,
		glecs_meta,
		name,
		"/",
		"/root/"
	);
}
