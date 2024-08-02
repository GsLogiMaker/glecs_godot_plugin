
#include "utils.h"

#include <flecs.h>

using namespace godot;

/// Converts a godot::Variant::Type to Flecs's closest Entity type
EntityResult Utils::variant_type_to_id(Variant::Type type) {
	switch (type) {
		case Variant::BOOL:
			return EntityResult::Ok(ecs_id(ecs_bool_t));
		case Variant::INT:
			return EntityResult::Ok(ecs_id(ecs_i64_t));
		
		default:
			return EntityResult::Err(
				String("Could not convert Variant type ")
				+ Variant::get_type_name(type)
				+ " to entity ID"
			);
	}
}
