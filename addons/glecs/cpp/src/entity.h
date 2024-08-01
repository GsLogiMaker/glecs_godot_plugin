
#ifndef GL_ENTITY_H
#define GL_ENTITY_H

#include "world.h"

#include <flecs.h>
#include <godot_cpp/classes/ref_counted.hpp>

namespace godot {

	// Forward declare GlComponent instead of including component.h to avoid
	// cyclic dependncies that causes GlComponent to compile before GlEntity
	// is defined.
	class GlComponent;

	class GlEntity : public RefCounted {
		GDCLASS(GlEntity, RefCounted)

	public:
		GlEntity();
		~GlEntity();

		static Ref<GlEntity> spawn(GlWorld*);
		static Ref<GlEntity> from(ecs_entity_t, GlWorld*);

		Ref<GlComponent> get_component(ecs_entity_t);
		
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
