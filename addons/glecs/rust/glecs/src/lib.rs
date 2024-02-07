
pub mod component;
pub mod entity;
pub mod world;

pub(crate) mod component_definitions;

use std::mem::size_of;

use godot::prelude::*;
use godot::engine::Object;

const TYPE_SIZES:&'static [usize] = &[
    /* NIL */ 0,
    /* BOOL */ 4, //size_of::<bool>(),
    /* INT */ size_of::<i32>(),
    /* FLOAT */ size_of::<f64>(),
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
    /* OBJECT */ size_of::<Object>(),
    /* CALLABLE */ size_of::<Callable>(),
    /* SIGNAL */ size_of::<Signal>(),
    /* DICTIONARY */ size_of::<Dictionary>(),
    /* ARRAY */ size_of::<Array<()>>(),
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

struct GECS; #[gdextension] unsafe impl ExtensionLibrary for GECS {}

