
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

		static GlWorld* singleton();
		ecs_world_t* raw();

	protected:
		static void _bind_methods();

	private:
		ecs_world_t* _raw;

	};
}

#endif
