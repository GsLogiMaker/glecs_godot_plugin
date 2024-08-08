
#ifndef WORLD_H
#define WORLD_H

#include "godot_cpp/variant/utility_functions.hpp"
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
		static ecs_entity_t variant_type_to_id(Variant::Type);
		static Variant::Type id_to_variant_type(ecs_entity_t);

		// **************************************
		// *** Unexposed ***
		// **************************************

		// *** Glecs entities ***
		static ecs_entity_t glecs;
		static ecs_entity_t glecs_meta;
		static ecs_entity_t glecs_meta_real;
		static ecs_entity_t glecs_meta_nil;
		static ecs_entity_t glecs_meta_bool;
		static ecs_entity_t glecs_meta_int;
		static ecs_entity_t glecs_meta_float;
		static ecs_entity_t glecs_meta_string;
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
		static ecs_entity_t glecs_meta_string_name;
		static ecs_entity_t glecs_meta_node_path;
		static ecs_entity_t glecs_meta_rid;
		static ecs_entity_t glecs_meta_object;
		static ecs_entity_t glecs_meta_callable;
		static ecs_entity_t glecs_meta_signal;
		static ecs_entity_t glecs_meta_dictionary;
		static ecs_entity_t glecs_meta_array;
		static ecs_entity_t glecs_meta_packed_byte_array;
		static ecs_entity_t glecs_meta_packed_int32_array;
		static ecs_entity_t glecs_meta_packed_int64_array;
		static ecs_entity_t glecs_meta_packed_float32_array;
		static ecs_entity_t glecs_meta_packed_float64_array;
		static ecs_entity_t glecs_meta_packed_string_array;
		static ecs_entity_t glecs_meta_packed_vector2_array;
		static ecs_entity_t glecs_meta_packed_vector3_array;
		static ecs_entity_t glecs_meta_packed_color_array;

		void copy_component_ptr(const void*, void*, ecs_entity_t);
		void copy_gd_type_ptr(const void*, void*, ecs_entity_t);
		void deinit_component_ptr(void*, ecs_entity_t);
		void deinit_gd_type_ptr(void*, ecs_entity_t);
		void init_component_ptr(void*, ecs_entity_t, Variant);
		void init_gd_type_ptr(void*, ecs_entity_t);

		static GlWorld* singleton();
		ecs_world_t* raw();

	protected:
		static void _bind_methods();

	private:
		ecs_world_t* _raw;

		template<typename T>
		static void gd_type_ctor(
			void* ptr,
			int32_t count,
			const ecs_type_info_t* type_info
		) {
			T* list = (T*)ptr;
			for (int i=0; i != count; i++) {
				T value = T();
				list[i] = value;
			}
		}

		template<typename T>
		static void gd_type_dtor(
			void* ptr,
			int32_t count,
			const ecs_type_info_t* type_info
		) {
			T* list = (T*)ptr;
			for (int i=0; i != count; i++) {
				list[i].~T();
			}
		}

		template<typename T>
		static void gd_type_copy(
			void* dst_ptr,
			const void* src_ptr,
			int32_t count,
			const ecs_type_info_t* type_info
		) {
			T* dst_list = (T*)dst_ptr;
			const T* src_list = (const T*)src_ptr;
			for (int i=0; i != count; i++) {
				dst_list[i] = T(src_list[i]);
			}
		}

		template<typename T>
		static void gd_type_move(
			void* dst_ptr,
			void* src_ptr,
			int32_t count,
			const ecs_type_info_t* type_info
		) {
			T* dst_list = (T*)dst_ptr;
			T* src_list = (T*)src_ptr;
			for (int i=0; i != count; i++) {
				dst_list[i] = T(src_list[i]);
			}
		}

		template<typename T>
		void define_gd_component(
			const char* name,
			ecs_entity_t* static_id
		) {
			ecs_component_desc_t desc = {
				.entity = *static_id,
				.type = {
					.size = sizeof(T),
					.alignment = 8
				}
			}; ecs_component_init(_raw, &desc);
			ecs_type_hooks_t hooks = {
				.ctor = GlWorld::gd_type_ctor<T>,
				.dtor = GlWorld::gd_type_dtor<T>,
				.copy = GlWorld::gd_type_copy<T>,
				.move = GlWorld::gd_type_move<T>
			}; ecs_set_hooks_id(_raw, *static_id, &hooks);
			ecs_add_path_w_sep(
				_raw,
				*static_id,
				glecs_meta,
				name,
				"/",
				"/root/"
			);
		}

		void define_gd_literal(const char*, ecs_primitive_kind_t, ecs_entity_t* id_storage);
	};
}

#endif
