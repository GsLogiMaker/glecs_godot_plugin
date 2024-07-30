
#ifndef GDEXAMPLE_H
#define GDEXAMPLE_H

#include "flecs/flecs.h"
#include <godot_cpp/classes/object.hpp>

namespace godot {

	class GFWorld : public Object {
		GDCLASS(GFWorld, Object)

	private:
		ecs_world_t *raw;

	protected:
		static void _bind_methods();

	public:
		GFWorld();
		~GFWorld();
	};

}

#endif