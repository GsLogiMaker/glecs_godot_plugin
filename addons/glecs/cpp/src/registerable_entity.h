
#ifndef REGISTERABLE_ENTITY_H
#define REGISTERABLE_ENTITY_H

#include "entity.h"

#include <flecs.h>
#include <godot_cpp/classes/ref_counted.hpp>

namespace godot {

	class GlRegisterableEntity : public GlEntity {
		GDCLASS(GlRegisterableEntity, GlEntity)

	public:
		GlRegisterableEntity();
		~GlRegisterableEntity();

	protected:
		static void _bind_methods();
	
	private:
	};

}

#endif
