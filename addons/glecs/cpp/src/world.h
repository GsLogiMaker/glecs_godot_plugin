
#ifndef WORLD_H
#define WORLD_H

#include <flecs.h>
#include <godot_cpp/classes/object.hpp>
#include <godot_cpp/classes/ref_counted.hpp>

namespace godot {

	// Predefine instead of include to avoid cyclic dependencies
	class GlComponentBuilder;

	class GlWorld : public Object {
		GDCLASS(GlWorld, Object)

	public:
		GlWorld();
		~GlWorld();

		// **************************************
		// *** Exposed ***
		// **************************************

		Ref<GlComponentBuilder> component_builder();

		ecs_entity_t coerce_id(Variant);
		void progress(double delta);
		void start_rest_api();

		// **************************************
		// *** Unexposed ***
		// **************************************

		// *** Glecs entities ***
		static ecs_entity_t glecs;
		static ecs_entity_t glecs_meta;
		static ecs_entity_t glecs_meta_real;
		static ecs_entity_t glecs_meta_vector2;
		static ecs_entity_t glecs_meta_vector2i;
		static ecs_entity_t glecs_meta_rect2;
		static ecs_entity_t glecs_meta_rect2i;
		static ecs_entity_t glecs_meta_vector3;
		static ecs_entity_t glecs_meta_vector3i;
		static ecs_entity_t glecs_meta_transform2d;
		static ecs_entity_t glecs_meta_vector4;
		static ecs_entity_t glecs_meta_vector4i;
		static ecs_entity_t glecs_meta_plane;
		static ecs_entity_t glecs_meta_quaternion;
		static ecs_entity_t glecs_meta_aabb;
		static ecs_entity_t glecs_meta_basis;
		static ecs_entity_t glecs_meta_transform3d;
		static ecs_entity_t glecs_meta_projection;
		static ecs_entity_t glecs_meta_color;

		static GlWorld* singleton();
		ecs_world_t* raw();

	protected:
		static void _bind_methods();

	private:
		ecs_world_t* _raw;

	};
}

#endif
