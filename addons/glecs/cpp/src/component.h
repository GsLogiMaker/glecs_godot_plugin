
#ifndef COMPONENT_H
#define COMPONENT_H

#include "entity.h"
#include "registerable_entity.h"

#include <flecs.h>
#include <godot_cpp/classes/ref_counted.hpp>
#include <godot_cpp/variant/string.hpp>

namespace godot {

	class GlComponent : public GlRegisterableEntity {
		GDCLASS(GlComponent, GlRegisterableEntity)

	public:
		GlComponent();
		~GlComponent();

		Ref<GlEntity> get_source_entity();
		ecs_entity_t get_source_id();
		Variant get_member(String);

		void set_source_id(ecs_entity_t id);

	protected:
		static void _bind_methods();

		void* get_member_at(int offset);
		const EcsMember* get_member_data(String);

	
	private:
		ecs_entity_t source_entity_id;
	};

}

#endif
