
#ifndef COMPONENT_Builder_H
#define COMPONENT_Builder_H

#include <flecs.h>
#include <godot_cpp/classes/ref_counted.hpp>
#include <godot_cpp/variant/string.hpp>

namespace godot {

	// Predefine instead of include to avoid cyclic dependencies
	class GlWorld;

	class GlComponentBuilder : public RefCounted {
		GDCLASS(GlComponentBuilder, RefCounted)

	public:
		GlComponentBuilder();
		~GlComponentBuilder();

		// **************************************
		// *** Exposed ***
		// **************************************

		Ref<GlComponentBuilder> add_member(String, Variant::Type);
		int get_member_count();
		Ref<GlComponentBuilder> set_name(String);
		void build();

		// **************************************
		// *** Unexposed ***
		// **************************************

		void set_world(GlWorld*);

	protected:
		static void _bind_methods();

	private:
		ecs_component_desc_t component_desc;
		ecs_struct_desc_t struct_desc;
		/// @brief A list of the allocated names of members
		/// Also used to find the number of added of members
		Array member_names;
		/// The name of this compoennt
		String name;
		GlWorld* world;
		/// Is true if this builder has already been built
		bool built;
	};

}

#endif
