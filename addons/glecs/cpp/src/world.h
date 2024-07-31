
#ifndef WORLD_H
#define WORLD_H

#include <flecs.h>
#include <godot_cpp/classes/object.hpp>

namespace godot {

	class GlWorld : public Object {
		GDCLASS(GlWorld, Object)

	public:
		GlWorld();
		~GlWorld();

		static GlWorld* singleton();

		ecs_world_t * raw();

	protected:
		static void _bind_methods();

	private:
		ecs_world_t *_raw;

	};
}

#endif
