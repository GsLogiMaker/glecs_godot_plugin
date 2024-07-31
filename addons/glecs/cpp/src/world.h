
#ifndef WORLD_H
#define WORLD_H

#include <flecs.h>
#include <godot_cpp/classes/object.hpp>

namespace godot {

	class GlWorld : public Object {
		GDCLASS(GlWorld, Object)

	private:
		ecs_world_t *_raw;

	protected:
		static void _bind_methods();

	public:
		GlWorld();
		~GlWorld();

		ecs_world_t * raw();
	};

}

#endif
