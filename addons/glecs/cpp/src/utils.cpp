
#include "utils.h"
#include "world.h"

#include <flecs.h>

using namespace godot;

/// Converts a godot::Variant::Type to Flecs's closest Entity type
EntityResult Utils::variant_type_to_id(Variant::Type type) {
	switch (type) {
		case Variant::BOOL: return EntityResult::Ok(ecs_id(ecs_bool_t));
		case Variant::INT: return EntityResult::Ok(ecs_id(ecs_i64_t));
		case Variant::FLOAT: return EntityResult::Ok(ecs_id(ecs_f64_t));
		case Variant::VECTOR2: return EntityResult::Ok(GlWorld::glecs_meta_vector2);
		case Variant::VECTOR2I: return EntityResult::Ok(GlWorld::glecs_meta_vector2i);
		case Variant::RECT2: return EntityResult::Ok(GlWorld::glecs_meta_rect2);
		case Variant::RECT2I: return EntityResult::Ok(GlWorld::glecs_meta_rect2i);
		case Variant::VECTOR3: return EntityResult::Ok(GlWorld::glecs_meta_vector3);
		case Variant::VECTOR3I: return EntityResult::Ok(GlWorld::glecs_meta_vector3i);
		case Variant::TRANSFORM2D: return EntityResult::Ok(GlWorld::glecs_meta_transform2d);
		case Variant::VECTOR4: return EntityResult::Ok(GlWorld::glecs_meta_vector4);
		case Variant::VECTOR4I: return EntityResult::Ok(GlWorld::glecs_meta_vector4i);
		case Variant::PLANE: return EntityResult::Ok(GlWorld::glecs_meta_plane);
		case Variant::QUATERNION: return EntityResult::Ok(GlWorld::glecs_meta_quaternion);
		case Variant::AABB: return EntityResult::Ok(GlWorld::glecs_meta_aabb);
		case Variant::BASIS: return EntityResult::Ok(GlWorld::glecs_meta_basis);
		case Variant::TRANSFORM3D: return EntityResult::Ok(GlWorld::glecs_meta_transform3d);
		case Variant::PROJECTION: return EntityResult::Ok(GlWorld::glecs_meta_projection);
		case Variant::COLOR: return EntityResult::Ok(GlWorld::glecs_meta_color);
		
		case Variant::STRING_NAME: return EntityResult::Ok(GlWorld::glecs_meta_string_name);
		case Variant::NODE_PATH: return EntityResult::Ok(GlWorld::glecs_meta_node_path);
		case Variant::RID: return EntityResult::Ok(GlWorld::glecs_meta_rid);
		case Variant::OBJECT: return EntityResult::Ok(GlWorld::glecs_meta_object);
		case Variant::CALLABLE: return EntityResult::Ok(GlWorld::glecs_meta_callable);
		case Variant::SIGNAL: return EntityResult::Ok(GlWorld::glecs_meta_signal);
		case Variant::DICTIONARY: return EntityResult::Ok(GlWorld::glecs_meta_dictionary);
		case Variant::ARRAY: return EntityResult::Ok(GlWorld::glecs_meta_array);
		case Variant::PACKED_INT32_ARRAY: return EntityResult::Ok(GlWorld::glecs_meta_packed_int32_array);
		case Variant::PACKED_INT64_ARRAY: return EntityResult::Ok(GlWorld::glecs_meta_packed_int64_array);
		case Variant::PACKED_FLOAT32_ARRAY: return EntityResult::Ok(GlWorld::glecs_meta_packed_float32_array);
		case Variant::PACKED_FLOAT64_ARRAY: return EntityResult::Ok(GlWorld::glecs_meta_packed_float64_array);
		case Variant::PACKED_STRING_ARRAY: return EntityResult::Ok(GlWorld::glecs_meta_packed_string_array);
		case Variant::PACKED_VECTOR2_ARRAY: return EntityResult::Ok(GlWorld::glecs_meta_packed_vector2_array);
		case Variant::PACKED_VECTOR3_ARRAY: return EntityResult::Ok(GlWorld::glecs_meta_packed_vector3_array);
		case Variant::PACKED_COLOR_ARRAY: return EntityResult::Ok(GlWorld::glecs_meta_packed_color_array);
		
		default:
			return EntityResult::Err(
				String("Could not convert Variant type ")
				+ Variant::get_type_name(type)
				+ " to entity ID"
			);
	}
}
