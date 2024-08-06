
#include "world.h"
#include "entity.h"
#include "component_builder.h"
#include "utils.h"

#include <flecs.h>
#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/classes/engine.hpp>
#include <godot_cpp/variant/string_name.hpp>

using namespace godot;

ecs_entity_t GlWorld::glecs = 0;
ecs_entity_t GlWorld::glecs_meta = 0;
ecs_entity_t GlWorld::glecs_meta_real = 0;
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

	{
		// Add glecs/meta/Real type
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

	{
		// Add glecs/meta/Vector2 type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Vector2"),
			.members = {
				{.name = "x", .type = glecs_meta_real},
				{.name = "y", .type = glecs_meta_real}
			}
		};
		glecs_meta_vector2 = ecs_struct_init(_raw, &desc);
	}

	{
		// Add glecs/meta/Vector2i type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Vector2i"),
			.members = {
				{.name = "x", .type = ecs_id(ecs_i32_t)},
				{.name = "y", .type = ecs_id(ecs_i32_t)}
			}
		};
		glecs_meta_vector2i = ecs_struct_init(_raw, &desc);
	}

	{
		// Add glecs/meta/Rect type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Rect2"),
			.members = {
				{.name = "position", .type = glecs_meta_vector2},
				{.name = "size", .type = glecs_meta_vector2}
			}
		};
		glecs_meta_rect2 = ecs_struct_init(_raw, &desc);
	}

	{
		// Add glecs/meta/Rect2i type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Rect2i"),
			.members = {
				{.name = "position", .type = glecs_meta_vector2i},
				{.name = "size", .type = glecs_meta_vector2i}
			}
		};
		glecs_meta_rect2i = ecs_struct_init(_raw, &desc);
	}

	{
		// Add glecs/meta/Vector3 type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Vector3"),
			.members = {
				{.name = "x", .type = glecs_meta_real},
				{.name = "y", .type = glecs_meta_real},
				{.name = "z", .type = glecs_meta_real}
			}
		};
		glecs_meta_vector3 = ecs_struct_init(_raw, &desc);
	}

	{
		// Add glecs/meta/Vector3i type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Vector3i"),
			.members = {
				{.name = "x", .type = ecs_id(ecs_i32_t)},
				{.name = "y", .type = ecs_id(ecs_i32_t)},
				{.name = "z", .type = ecs_id(ecs_i32_t)}
			}
		};
		glecs_meta_vector3i = ecs_struct_init(_raw, &desc);
	}

	{
		// Add glecs/meta/Transform2D type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Transform2D"),
			.members = {
				{.name = "x", .type = glecs_meta_vector2},
				{.name = "y", .type = glecs_meta_vector2},
				{.name = "origin", .type = glecs_meta_vector2}
			}
		};
		glecs_meta_transform2d = ecs_struct_init(_raw, &desc);
	}

	{
		// Add glecs/meta/Vector4 type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Vector4"),
			.members = {
				{.name = "x", .type = glecs_meta_real},
				{.name = "y", .type = glecs_meta_real},
				{.name = "z", .type = glecs_meta_real},
				{.name = "w", .type = glecs_meta_real}
			}
		};
		glecs_meta_vector4 = ecs_struct_init(_raw, &desc);
	}

	{
		// Add glecs/meta/Vector4i type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Vector4i"),
			.members = {
				{.name = "x", .type = ecs_id(ecs_i32_t)},
				{.name = "y", .type = ecs_id(ecs_i32_t)},
				{.name = "z", .type = ecs_id(ecs_i32_t)},
				{.name = "w", .type = ecs_id(ecs_i32_t)}
			}
		};
		glecs_meta_vector4i = ecs_struct_init(_raw, &desc);
	}

	{
		// Add glecs/meta/Plane type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Plane"),
			.members = {
				{.name = "x", .type = glecs_meta_real},
				{.name = "y", .type = glecs_meta_real},
				{.name = "z", .type = glecs_meta_real},
				{.name = "d", .type = glecs_meta_real},
				{.name = "normal", .type = glecs_meta_vector3}
			}
		};
		glecs_meta_plane = ecs_struct_init(_raw, &desc);
	}

	{
		// Add glecs/meta/Quaternion type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Quaternion"),
			.members = {
				{.name = "x", .type = glecs_meta_real},
				{.name = "y", .type = glecs_meta_real},
				{.name = "z", .type = glecs_meta_real},
				{.name = "w", .type = glecs_meta_real}
			}
		};
		glecs_meta_quaternion = ecs_struct_init(_raw, &desc);
	}

	{
		// Add glecs/meta/AABB type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "AABB"),
			.members = {
				{.name = "position", .type = glecs_meta_vector3},
				{.name = "size", .type = glecs_meta_vector3}
			}
		};
		glecs_meta_aabb = ecs_struct_init(_raw, &desc);
	}

	{
		// Add glecs/meta/Basis type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Basis"),
			.members = {
				{.name = "x", .type = glecs_meta_vector3},
				{.name = "y", .type = glecs_meta_vector3},
				{.name = "z", .type = glecs_meta_vector3}
			}
		};
		glecs_meta_basis = ecs_struct_init(_raw, &desc);
	}

	{
		// Add glecs/meta/Transform3D type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Transform3D"),
			.members = {
				{.name = "basis", .type = glecs_meta_basis},
				{.name = "origin", .type = glecs_meta_vector3}
			}
		};
		glecs_meta_transform3d = ecs_struct_init(_raw, &desc);
	}

	{
		// Add glecs/meta/Projection type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Projection"),
			.members = {
				{.name = "x", .type = glecs_meta_vector4},
				{.name = "y", .type = glecs_meta_vector4},
				{.name = "z", .type = glecs_meta_vector4},
				{.name = "w", .type = glecs_meta_vector4}
			}
		};
		glecs_meta_projection = ecs_struct_init(_raw, &desc);
	}

	{
		// Add glecs/meta/Color type
		ecs_struct_desc_t desc = {
			.entity = ecs_new_from_path(_raw, glecs_meta, "Color"),
			.members = {
				{.name = "r", .type = ecs_id(ecs_f32_t)},
				{.name = "g", .type = ecs_id(ecs_f32_t)},
				{.name = "b", .type = ecs_id(ecs_f32_t)},
				{.name = "a", .type = ecs_id(ecs_f32_t)}
			}
		};
		glecs_meta_color = ecs_struct_init(_raw, &desc);
	}

	define_gd_component_with_hooks<StringName>("StringName", &glecs_meta_string_name);
	define_gd_component_with_hooks<NodePath>("NodePath", &glecs_meta_node_path);
	define_gd_component_with_hooks<RID>("RID", &glecs_meta_rid);
	define_gd_component_with_hooks<Variant>("Object", &glecs_meta_object);
	define_gd_component_with_hooks<Callable>("Callable", &glecs_meta_callable);
	define_gd_component_with_hooks<Signal>("Signal", &glecs_meta_signal);
	define_gd_component_with_hooks<Variant>("Dictionary", &glecs_meta_dictionary);
	define_gd_component_with_hooks<Variant>("Array", &glecs_meta_array);
	define_gd_component_with_hooks<PackedInt32Array>("PackedInt32Array", &glecs_meta_packed_int32_array);
	define_gd_component_with_hooks<PackedInt64Array>("PackedInt64Array", &glecs_meta_packed_int64_array);
	define_gd_component_with_hooks<PackedFloat32Array>("PackedFloat32Array", &glecs_meta_packed_float32_array);
	define_gd_component_with_hooks<PackedFloat64Array>("PackedFloat64Array", &glecs_meta_packed_float64_array);
	define_gd_component_with_hooks<PackedStringArray>("PackedStringArray", &glecs_meta_packed_string_array);
	define_gd_component_with_hooks<PackedVector2Array>("PackedVector2Array", &glecs_meta_packed_vector2_array);
	define_gd_component_with_hooks<PackedVector3Array>("PackedVector3Array", &glecs_meta_packed_vector3_array);
	define_gd_component_with_hooks<PackedColorArray>("PackedColorArray", &glecs_meta_packed_color_array);

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

ecs_world_t * GlWorld::raw() {
	return _raw;
}

void GlWorld::_bind_methods() {
	godot::ClassDB::bind_method(D_METHOD("component_builder"), &GlWorld::component_builder);
	godot::ClassDB::bind_method(D_METHOD("coerce_id", "entity"), &GlWorld::coerce_id);
	godot::ClassDB::bind_method(D_METHOD("start_rest_api"), &GlWorld::start_rest_api);
	godot::ClassDB::bind_method(D_METHOD("progress", "delta"), &GlWorld::progress);
}
