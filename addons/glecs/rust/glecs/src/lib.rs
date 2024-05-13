
pub mod component;
pub mod entity;
pub mod event;
pub mod gd_bindings;
pub mod module;
pub mod prefab;
pub mod queries;
pub mod util;
pub mod world;

pub(crate) mod component_definitions;

use std::mem::size_of;

use godot::prelude::*;

/// Godot's native int type
type Int = i64;
/// Godot's native float type
type Float = f64;

const TYPE_SIZES:&'static [usize] = &[
    /* NIL */ 0,
    /* BOOL */ 4, //size_of::<bool>(),
    /* INT */ size_of::<Int>(),
    /* FLOAT */ size_of::<Float>(),
    /* STRING */ size_of::<String>(),
    /* VECTOR2 */ size_of::<Vector2>(),
    /* VECTOR2I */ size_of::<Vector2i>(),
    /* RECT2 */ size_of::<Rect2>(),
    /* RECT2I */ size_of::<Rect2i>(),
    /* VECTOR3 */ size_of::<Vector3>(),
    /* VECTOR3I */ size_of::<Vector3i>(),
    /* TRANSFORM2D */ size_of::<Transform2D>(),
    /* VECTOR4 */ size_of::<Vector4>(),
    /* VECTOR4I */ size_of::<Vector4i>(),
    /* PLANE */ size_of::<Plane>(),
    /* QUATERNION */ size_of::<Quaternion>(),
    /* AABB */ size_of::<Aabb>(),
    /* BASIS */ size_of::<Basis>(),
    /* TRANSFORM3D */ size_of::<Transform3D>(),
    /* PROJECTION */ size_of::<Projection>(),
    /* COLOR */ size_of::<Color>(),
    /* STRING_NAME */ size_of::<StringName>(),
    /* NODE_PATH */ size_of::<NodePath>(),
    /* RID */ size_of::<Rid>(),
    /* OBJECT */ size_of::<Variant>(), // Objects are stored in components as Variant
    /* CALLABLE */ size_of::<Callable>(),
    /* SIGNAL */ size_of::<Signal>(),
    /* DICTIONARY */ size_of::<Variant>(), // Dictionaries are stored in components as Variant
    /* ARRAY */ size_of::<Variant>(), // Arrays are stored in components as Variant
    /* PACKED_BYTE_ARRAY */ size_of::<PackedByteArray>(),
    /* PACKED_INT32_ARRAY */ size_of::<PackedInt32Array>(),
    /* PACKED_INT64_ARRAY */ size_of::<PackedInt64Array>(),
    /* PACKED_FLOAT32_ARRAY */ size_of::<PackedFloat32Array>(),
    /* PACKED_FLOAT64_ARRAY */ size_of::<PackedFloat64Array>(),
    /* PACKED_STRING_ARRAY */ size_of::<PackedStringArray>(),
    /* PACKED_VECTOR2_ARRAY */ size_of::<PackedVector2Array>(),
    /* PACKED_VECTOR3_ARRAY */ size_of::<PackedVector3Array>(),
    /* PACKED_COLOR_ARRAY */ size_of::<PackedColorArray>(),
    /* MAX */ 0,
];

struct Glecs; #[gdextension] unsafe impl ExtensionLibrary for Glecs {}

#[macro_export]
macro_rules! show_error {
    ($title:literal, $fmt:literal $(, $args:expr)* $(,)?) => {
        {
            let msg = format!("***{}*** {}", $title, format!($fmt, $($args,)*));
            // godot_error!("{msg}");
            // godot_print!("{msg}");
            panic!("{}", msg);
        }
    };
}

#[cfg(test)]
mod test {
    use std::ffi::CStr;

    use flecs::*;
    use cstr::cstr;

    #[test]
    fn test_lookup_1_though_8_bug() {
        unsafe {
            let world = ecs_init();

            // Assert entity ID of 1 is component
            let comp_name = CStr::from_ptr(ecs_get_name(world, 1));
            dbg!(comp_name == cstr!(b"Component"));

            // Assert entity ID of 1 is component
            let flecs_name = CStr::from_ptr(ecs_get_name(world, 257));
            dbg!(flecs_name == cstr!(b"flecs"));

            // Create new entity named "1"
            let one = ecs_set_name(world, 0, cstr!(b"1").as_ptr());
            // Expect "1" to be a different entity to "Component"
            dbg!(one != 1); // false (bug)
        }
    }

    #[test]
    fn test_grow_into_number_name() {
        unsafe {
            let world = ecs_init();

            let e = ecs_new_id(world);
            ecs_set_name(world, e, cstr!(b"530").as_ptr());
            assert!(e == 525);
            
            let found = ecs_lookup(world, cstr!("Hello").as_ptr());
            assert!(found == 525);
        }
    }

    #[test]
    fn test_number_name_path() {
        unsafe {
            let world = ecs_init();

            let parent = ecs_new_id(world);
            ecs_set_name(world, parent, cstr!(b"MyParent").as_ptr());

            let child = ecs_new_id(world);
            ecs_add_id(world, child, ecs_pair(EcsChildOf, parent));
            ecs_set_name(world, child, cstr!(b"500").as_ptr());
            
            let found = ecs_lookup_path_w_sep(
                world,
                0,
                cstr!("MyParent/500").as_ptr(),
                cstr!("/").as_ptr(),
                cstr!("").as_ptr(),
                false,
            );
            assert!(found == child);
        }
    }
}