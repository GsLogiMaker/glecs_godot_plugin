
#ifndef ENTITY_H
#define ENTITY_H

#include "world.h"

#include <flecs.h>
#include <godot_cpp/classes/ref_counted.hpp>

namespace godot {

	class GlEntity : public RefCounted {
		GDCLASS(GlEntity, RefCounted)

	public:
		GlEntity();
		~GlEntity();

		static Ref<GlEntity> spawn(GlWorld*);
		static Ref<GlEntity> from(ecs_entity_t, GlWorld*);

		bool is_alive();

		ecs_entity_t get_id();
		GlWorld* get_world();

		void set_id(ecs_entity_t);
		void set_world(GlWorld*);

	protected:
		static void _bind_methods();
	
	private:
		ecs_entity_t id;
		GlWorld* world;
	};

}

#endif
