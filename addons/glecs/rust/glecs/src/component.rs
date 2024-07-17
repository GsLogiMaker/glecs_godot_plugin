
use std::ffi::c_void;
use std::fmt::Debug;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::rc::Rc;
use std::mem::size_of;

use flecs::EntityId;
use godot::prelude::*;

use crate::component_definitions::ComponetDefinition;
use crate::component_definitions::ComponetProperty;
use crate::entity::EntityLike;
use crate::gd_bindings::_GlecsBindings;
use crate::gd_bindings::_GlecsComponents;
use crate::show_error;
use crate::world::_GlecsBaseWorld;
use crate::Float;
use crate::Int;

/// An ECS component.
#[derive(GodotClass)]
#[class(base=RefCounted, no_init)]
pub struct _GlecsBaseComponent {
    pub(crate) base: Base<RefCounted>,
    pub(crate) world: Gd<_GlecsBaseWorld>,
    /// The ID that this component is attatached to.
    pub(crate) entity_id: EntityId,
    pub(crate) component_id: EntityId,
    pub(crate) component_definition: Rc<ComponetDefinition>,
}
#[godot_api]
impl _GlecsBaseComponent {
    /// Copies the data from the given component to this one.
    #[func]
    fn _copy_from_component(&mut self, from_component:Gd<_GlecsBaseComponent>) {
        EntityLike::validate(self);

        if self.get_flecs_id() != from_component.bind().get_flecs_id() {
            show_error!(
                "Failed to copy component",
                "Destination component is of type {}, but source component is of type {}",
                self.base().get_script(),
                from_component.bind().base().get_script(),
            )
        }
        unsafe {
            std::slice::from_raw_parts_mut(
                self.get_data().as_mut(),
                self.component_definition.layout.size(),
            ).copy_from_slice(
                std::slice::from_raw_parts(
                    from_component.bind().get_data().as_ptr(),
                    self.component_definition.layout.size(),
                ),
            );
        }
    }

    /// Returns the name of the the type of this component.
    #[func]
    fn _get_type_name(&self) -> StringName {
        EntityLike::validate(self);

        self.component_definition.name.clone()
    }

    /// Returns a property from the component data.
    #[func]
    fn _getc(&self, property: StringName) -> Variant {
        EntityLike::validate(self);

        let v = self._get_property(property.clone());
        v
    }

    /// Sets a property in the component data.
    #[func]
    fn _setc(&mut self, property: StringName, value:Variant) {
        EntityLike::validate(self);

        if !self._set_property(property.clone(), value.clone()) {
            show_error!(
                "Failed to set property",
                "No property named \"{}\" in component of type \"{}\"",
                property,
                self._get_type_name(),
            );
        }

        // Emit custom on set event
        _GlecsComponents::emit_on_set(
            self.world.clone(),
            self.entity_id,
            self.component_id,
        );
    }

    #[func]
    fn _delete(&self) {
        EntityLike::delete(self)
    }

    /// Override default 'free' behavior (This only works if the
    /// variable is staticly typed in GdScript.)
    #[func]
    fn free(&self) {
        EntityLike::delete(self)
    }

    #[func]
    fn _is_valid(&self) -> bool {
        EntityLike::is_valid(self)
    }

    pub(crate) fn create_initial_data(def: &ComponetDefinition, parameters:Variant) -> Box<[u8]> {
        let mut data = Vec::<u8>::new();
        data.resize(def.layout.size(), 0);
        
        match parameters.get_type() {
            VariantType::ARRAY => {
                let parameters = parameters.to::<VariantArray>();
                for
                    (i, property_meta)
                in def.parameters.iter().enumerate() {
                    let prop_value = if i < parameters.len() {
                        // Get value from passed parameters
                        let parameter = parameters.at(i);
                        let value = if
                            parameter.get_type() == property_meta.gd_type_id
                        {
                            parameter
                        } else {
                            // Parameter is wrong type, get value
                            // from component's default
                            def.get_property_default_value(
                                property_meta.name.to_variant(),
                            )
                        };
                        value
                    } else {
                        // Get value from component's default
                        def.get_property_default_value(
                            property_meta.name.to_variant(),
                        )
                    };

                    let nonnull_data = unsafe {
                        NonNull::new_unchecked(data.as_mut_ptr())
                    };
                    Self::init_property_data(nonnull_data, prop_value, &property_meta)
                }
            },
            VariantType::NIL => {
                for property_meta in def.parameters.iter() {
                    let default = def.get_property_default_value(
                        property_meta.name.to_variant(),
                    );
                    let nonnull_data = unsafe {
                        NonNull::new_unchecked(data.as_mut_ptr())
                    };
                    Self::init_property_data(nonnull_data, default, &property_meta)
                }
            },
            _ => todo!(),
        }
    
        data.into_boxed_slice()
    }

    // --- Getting ---

    pub(crate) fn _get_property(
		&self,
		property:StringName,
	) -> Variant {
        let Some(property_meta) = self
            .component_definition
            .get_property(&property)
            else {
                return Variant::nil();
            };
        
        let value = Self::get_property_data(
            self.get_data(),
            &property_meta,
        );

        if
            value == Variant::nil()
            && property_meta.gd_type_id != VariantType::OBJECT
        {
            show_error!(
                "Failed to get property",
                "No property named \"{}\" in component of type \"{}\"",
                property,
                self._get_type_name(),
            );
        }

        value
    }

    pub(crate) fn get_property_data(
        data: NonNull<u8>,
        property_data: &ComponetProperty,
    ) -> Variant{
        match property_data.gd_type_id {
            VariantType::NIL => panic!("Can't set \"Nil\" type in component"),
            VariantType::BOOL => Self::get_property_data_raw::<bool>(data, property_data.offset).to_variant(),
            VariantType::INT => Self::get_property_data_raw::<Int>(data, property_data.offset).to_variant(),
            VariantType::FLOAT => Self::get_property_data_raw::<Float>(data, property_data.offset).to_variant(),
            VariantType::STRING => Self::get_property_data_raw::<GString>(data, property_data.offset).to_variant(),
            VariantType::VECTOR2 => Self::get_property_data_raw::<Vector2>(data, property_data.offset).to_variant(),
            VariantType::VECTOR2I => Self::get_property_data_raw::<Vector2i>(data, property_data.offset).to_variant(),
            VariantType::RECT2 => Self::get_property_data_raw::<Rect2>(data, property_data.offset).to_variant(),
            VariantType::RECT2I => Self::get_property_data_raw::<Rect2i>(data, property_data.offset).to_variant(),
            VariantType::VECTOR3 => Self::get_property_data_raw::<Vector3>(data, property_data.offset).to_variant(),
            VariantType::VECTOR3I => Self::get_property_data_raw::<Vector3i>(data, property_data.offset).to_variant(),
            VariantType::TRANSFORM2D => Self::get_property_data_raw::<Transform2D>(data, property_data.offset).to_variant(),
            VariantType::VECTOR4 => Self::get_property_data_raw::<Vector4>(data, property_data.offset).to_variant(),
            VariantType::VECTOR4I => Self::get_property_data_raw::<Vector4i>(data, property_data.offset).to_variant(),
            VariantType::PLANE => Self::get_property_data_raw::<Plane>(data, property_data.offset).to_variant(),
            VariantType::QUATERNION => Self::get_property_data_raw::<Quaternion>(data, property_data.offset).to_variant(),
            VariantType::AABB => Self::get_property_data_raw::<Aabb>(data, property_data.offset).to_variant(),
            VariantType::BASIS => Self::get_property_data_raw::<Basis>(data, property_data.offset).to_variant(),
            VariantType::TRANSFORM3D => Self::get_property_data_raw::<Transform3D>(data, property_data.offset).to_variant(),
            VariantType::PROJECTION => Self::get_property_data_raw::<Projection>(data, property_data.offset).to_variant(),
            VariantType::COLOR => Self::get_property_data_raw::<Color>(data, property_data.offset).to_variant(),
            VariantType::STRING_NAME => Self::get_property_data_raw::<StringName>(data, property_data.offset).to_variant(),
            VariantType::NODE_PATH => Self::get_property_data_raw::<NodePath>(data, property_data.offset).to_variant(),
            VariantType::RID => Self::get_property_data_raw::<Rid>(data, property_data.offset).to_variant(),
            VariantType::OBJECT => Self::get_property_data_raw_variant(data, property_data.offset).to_variant(),
            VariantType::CALLABLE => Self::get_property_data_raw::<Callable>(data, property_data.offset).to_variant(),
            VariantType::SIGNAL => Self::get_property_data_raw::<Signal>(data, property_data.offset).to_variant(),
            VariantType::DICTIONARY => Self::get_property_data_raw_variant(data, property_data.offset).to_variant(),
            VariantType::ARRAY => Self::get_property_data_raw_variant(data, property_data.offset).to_variant(),
            VariantType::PACKED_BYTE_ARRAY => Self::get_property_data_raw::<PackedByteArray>(data, property_data.offset).to_variant(),
            VariantType::PACKED_INT32_ARRAY => Self::get_property_data_raw::<PackedInt32Array>(data, property_data.offset).to_variant(),
            VariantType::PACKED_INT64_ARRAY => Self::get_property_data_raw::<PackedInt64Array>(data, property_data.offset).to_variant(),
            VariantType::PACKED_FLOAT32_ARRAY => Self::get_property_data_raw::<PackedFloat32Array>(data, property_data.offset).to_variant(),
            VariantType::PACKED_FLOAT64_ARRAY => Self::get_property_data_raw::<PackedFloat64Array>(data, property_data.offset).to_variant(),
            VariantType::PACKED_STRING_ARRAY => Self::get_property_data_raw::<PackedStringArray>(data, property_data.offset).to_variant(),
            VariantType::PACKED_VECTOR2_ARRAY => Self::get_property_data_raw::<PackedVector2Array>(data, property_data.offset).to_variant(),
            VariantType::PACKED_VECTOR3_ARRAY => Self::get_property_data_raw::<PackedVector3Array>(data, property_data.offset).to_variant(),
            VariantType::PACKED_COLOR_ARRAY => Self::get_property_data_raw::<PackedColorArray>(data, property_data.offset).to_variant(),
            _ => unreachable!(),
        }
    }
    
    pub(crate) fn get_property_data_raw<T: Clone + Debug>(
        data: NonNull<u8>,
        offset: usize,
    ) -> T {
        let prop_ptr = unsafe {
            NonNull::new_unchecked(data.as_ptr().add(offset))
        };
        let casted_value = unsafe {
            prop_ptr.cast::<ManuallyDrop<T>>()
                .as_ref()
        };
        ManuallyDrop::into_inner(casted_value.clone())
    }
    
    fn get_property_data_raw_variant(
        data: NonNull<u8>,
        offset: usize,
    ) -> Variant {
        let prop_ptr = unsafe {
            NonNull::new_unchecked(data.as_ptr().add(offset))
        };
        let got_value = unsafe {
            prop_ptr.cast::<ManuallyDrop<Variant>>()
                .as_ref()
        };
        ManuallyDrop::into_inner(got_value.clone())
    }

    // --- Setting ---

    pub(crate) fn _set_property(
		&mut self,
		property:StringName,
		value:Variant,
	) -> bool {
        let Some(property_meta) = self
            .component_definition
            .get_property(&property) else {
                return false;
            };

        let value_type = value.get_type();
        let property_type = property_meta.gd_type_id;
        'cancel_type_check: {
            if property_type == VariantType::NIL {
                break 'cancel_type_check
            }
            if value_type != property_type {
                if
                    property_type == VariantType::OBJECT
                        && value_type == VariantType::NIL
                { break 'cancel_type_check }

                show_error!(
                    "Failed to set property",
                    "Expected type {:?}, but got type {:?}.",
                    property_type,
                    value_type,
                );
                // return true;
            }
        }
        
        Self::set_property_data(self.get_data(), value, &property_meta);

        return true;
    }

    // Sets the property of the given data to it's type from a variant
    pub(crate) fn set_property_data(
        data: NonNull<u8>,
        value: Variant,
        property_data: &ComponetProperty,
    ) {
        match property_data.gd_type_id {
            VariantType::NIL => panic!("Can't set \"Nil\" type in component"),
            VariantType::BOOL => Self::set_property_data_raw::<bool>(data, value, property_data.offset),
            VariantType::INT => Self::set_property_data_raw::<Int>(data, value, property_data.offset),
            VariantType::FLOAT => Self::set_property_data_raw::<Float>(data, value, property_data.offset),
            VariantType::STRING => Self::set_property_data_raw::<GString>(data, value, property_data.offset),
            VariantType::VECTOR2 => Self::set_property_data_raw::<Vector2>(data, value, property_data.offset),
            VariantType::VECTOR2I => Self::set_property_data_raw::<Vector2i>(data, value, property_data.offset),
            VariantType::RECT2 => Self::set_property_data_raw::<Rect2>(data, value, property_data.offset),
            VariantType::RECT2I => Self::set_property_data_raw::<Rect2i>(data, value, property_data.offset),
            VariantType::VECTOR3 => Self::set_property_data_raw::<Vector3>(data, value, property_data.offset),
            VariantType::VECTOR3I => Self::set_property_data_raw::<Vector3i>(data, value, property_data.offset),
            VariantType::TRANSFORM2D => Self::set_property_data_raw::<Transform2D>(data, value, property_data.offset),
            VariantType::VECTOR4 => Self::set_property_data_raw::<Vector4>(data, value, property_data.offset),
            VariantType::VECTOR4I => Self::set_property_data_raw::<Vector4i>(data, value, property_data.offset),
            VariantType::PLANE => Self::set_property_data_raw::<Plane>(data, value, property_data.offset),
            VariantType::QUATERNION => Self::set_property_data_raw::<Quaternion>(data, value, property_data.offset),
            VariantType::AABB => Self::set_property_data_raw::<Aabb>(data, value, property_data.offset),
            VariantType::BASIS => Self::set_property_data_raw::<Basis>(data, value, property_data.offset),
            VariantType::TRANSFORM3D => Self::set_property_data_raw::<Transform3D>(data, value, property_data.offset),
            VariantType::PROJECTION => Self::set_property_data_raw::<Projection>(data, value, property_data.offset),
            VariantType::COLOR => Self::set_property_data_raw::<Color>(data, value, property_data.offset),
            VariantType::STRING_NAME => Self::set_property_data_raw::<StringName>(data, value, property_data.offset),
            VariantType::NODE_PATH => Self::set_property_data_raw::<NodePath>(data, value, property_data.offset),
            VariantType::RID => Self::set_property_data_raw::<Rid>(data, value, property_data.offset),
            VariantType::OBJECT => Self::set_property_data_raw_variant(data, value, property_data.offset),
            VariantType::CALLABLE => Self::set_property_data_raw::<Callable>(data, value, property_data.offset),
            VariantType::SIGNAL => Self::set_property_data_raw::<Signal>(data, value, property_data.offset),
            VariantType::DICTIONARY => Self::set_property_data_raw_variant(data, value, property_data.offset),
            VariantType::ARRAY => Self::set_property_data_raw_variant(data, value, property_data.offset),
            VariantType::PACKED_BYTE_ARRAY => Self::set_property_data_raw::<PackedByteArray>(data, value, property_data.offset),
            VariantType::PACKED_INT32_ARRAY => Self::set_property_data_raw::<PackedInt32Array>(data, value, property_data.offset),
            VariantType::PACKED_INT64_ARRAY => Self::set_property_data_raw::<PackedInt64Array>(data, value, property_data.offset),
            VariantType::PACKED_FLOAT32_ARRAY => Self::set_property_data_raw::<PackedFloat32Array>(data, value, property_data.offset),
            VariantType::PACKED_FLOAT64_ARRAY => Self::set_property_data_raw::<PackedFloat64Array>(data, value, property_data.offset),
            VariantType::PACKED_STRING_ARRAY => Self::set_property_data_raw::<PackedStringArray>(data, value, property_data.offset),
            VariantType::PACKED_VECTOR2_ARRAY => Self::set_property_data_raw::<PackedVector2Array>(data, value, property_data.offset),
            VariantType::PACKED_VECTOR3_ARRAY => Self::set_property_data_raw::<PackedVector3Array>(data, value, property_data.offset),
            VariantType::PACKED_COLOR_ARRAY => Self::set_property_data_raw::<PackedColorArray>(data, value, property_data.offset),
            _ => unreachable!(),
        }
    }
    
    pub(crate) fn set_property_data_raw<T: FromGodot + Debug>(
        data: NonNull<u8>,
        value: Variant,
        offset: usize,
    ) {
        let prop_ptr = unsafe {
            NonNull::new_unchecked(data.as_ptr().add(offset))
        };
        let prop_mut = unsafe { prop_ptr.cast::<ManuallyDrop<T>>().as_mut() };
        let new_value = ManuallyDrop::new(value.to::<T>());
        let mut old_prop = std::mem::replace(prop_mut, new_value);
        drop(unsafe { ManuallyDrop::take(&mut old_prop) })
    }

    fn set_property_data_raw_variant(
        data: NonNull<u8>,
        new_value: Variant,
        offset: usize,
    ) {
        let prop_ptr = unsafe {
            NonNull::new_unchecked(data.as_ptr().add(offset))
        };
        let prop_mut = unsafe {
            prop_ptr.cast::<ManuallyDrop<Variant>>().as_mut()
        };
        let mut old_prop = std::mem::replace(
            prop_mut,
            ManuallyDrop::new(new_value),
        );
        drop(unsafe { ManuallyDrop::take(&mut old_prop) })
    }

    // --- Initialization ---

    pub(crate) fn init_component_data(
        comp_data: NonNull<u8>,
        comp_def: &ComponetDefinition,
    ) {
        for p in comp_def.parameters.iter() {
            let initial_value = comp_def
                .get_property_default_value(p.name.to_variant());
            Self::init_property_data(comp_data, initial_value, p);
        }
    }

    // Similar to `[_set_property]`, except it does not call the destructor.
    pub(crate) fn _initialize_property(
        data: NonNull<u8>,
        description: &ComponetDefinition,
        property: StringName,
        value: Variant,
    ) -> bool {
        let Some(property_data) = description
            .get_property(&property) else {
                show_error!(
                    "Property initialization failed",
                    "Can't write to {} in {{component}}. Component has no property with that name",
                    property,
                );
                // return false;
            };

        let value_type = value.get_type();
        let property_type = property_data.gd_type_id;
        if property_type != VariantType::NIL {
            if value_type != property_type && value_type != VariantType::NIL {
                show_error!(
                    "Failed to set property",
                    "Expected type {:?}, but got type {:?}.",
                    property_type,
                    value_type,
                );
                // return true;
            }
        }

        Self::init_property_data(data, value, property_data);

        return true;
    }

    pub(crate) fn init_property_data(
        data: NonNull<u8>,
        value: Variant,
        property_data: &ComponetProperty,
    ) {
        match property_data.gd_type_id {
            VariantType::NIL => panic!("Can't init \"Nil\" type in component"),
            VariantType::BOOL => Self::init_property_data_raw::<bool>(data, value, property_data, &|| bool::default().to_variant()),
            VariantType::INT => Self::init_property_data_raw::<Int>(data, value, property_data, &|| Int::default().to_variant()),
            VariantType::FLOAT => Self::init_property_data_raw::<Float>(data, value, property_data, &|| Float::default().to_variant()),
            VariantType::STRING => Self::init_property_data_raw::<GString>(data, value, property_data, &|| GString::default().to_variant()),
            VariantType::VECTOR2 => Self::init_property_data_raw::<Vector2>(data, value, property_data, &|| Vector2::default().to_variant()),
            VariantType::VECTOR2I => Self::init_property_data_raw::<Vector2i>(data, value, property_data, &|| Vector2i::default().to_variant()),
            VariantType::RECT2 => Self::init_property_data_raw::<Rect2>(data, value, property_data, &|| Rect2::default().to_variant()),
            VariantType::RECT2I => Self::init_property_data_raw::<Rect2i>(data, value, property_data, &|| Rect2i::default().to_variant()),
            VariantType::VECTOR3 => Self::init_property_data_raw::<Vector3>(data, value, property_data, &|| Vector3::default().to_variant()),
            VariantType::VECTOR3I => Self::init_property_data_raw::<Vector3i>(data, value, property_data, &|| Vector3i::default().to_variant()),
            VariantType::TRANSFORM2D => Self::init_property_data_raw::<Transform2D>(data, value, property_data, &|| Transform2D::default().to_variant()),
            VariantType::VECTOR4 => Self::init_property_data_raw::<Vector4>(data, value, property_data, &|| Vector4::default().to_variant()),
            VariantType::VECTOR4I => Self::init_property_data_raw::<Vector4i>(data, value, property_data, &|| Vector4i::default().to_variant()),
            VariantType::PLANE => Self::init_property_data_raw::<Plane>(data, value, property_data, &|| Plane::invalid().to_variant()),
            VariantType::QUATERNION => Self::init_property_data_raw::<Quaternion>(data, value, property_data, &|| Quaternion::default().to_variant()),
            VariantType::AABB => Self::init_property_data_raw::<Aabb>(data, value, property_data, &|| Aabb::default().to_variant()),
            VariantType::BASIS => Self::init_property_data_raw::<Basis>(data, value, property_data, &|| Basis::default().to_variant()),
            VariantType::TRANSFORM3D => Self::init_property_data_raw::<Transform3D>(data, value, property_data, &|| Transform3D::default().to_variant()),
            VariantType::PROJECTION => Self::init_property_data_raw::<Projection>(data, value, property_data, &|| Projection::default().to_variant()),
            VariantType::COLOR => Self::init_property_data_raw::<Color>(data, value, property_data, &|| Color::default().to_variant()),
            VariantType::STRING_NAME => Self::init_property_data_raw::<StringName>(data, value, property_data, &|| StringName::default().to_variant()),
            VariantType::NODE_PATH => Self::init_property_data_raw::<NodePath>(data, value, property_data, &|| NodePath::default().to_variant()),
            VariantType::RID => Self::init_property_data_raw::<Rid>(data, value, property_data, &|| Rid::new(0).to_variant()),
            VariantType::OBJECT => Self::init_property_data_raw_variant(data, value, property_data),
            VariantType::CALLABLE => Self::init_property_data_raw::<Callable>(data, value, property_data, &|| Callable::invalid().to_variant()),
            VariantType::SIGNAL => Self::init_property_data_raw::<Signal>(data, value, property_data, &|| Signal::invalid().to_variant()),
            VariantType::DICTIONARY => Self::init_property_data_raw_variant(data, value, property_data),
            VariantType::ARRAY => Self::init_property_data_raw_variant(data, value, property_data),
            VariantType::PACKED_BYTE_ARRAY => Self::init_property_data_raw::<PackedByteArray>(data, value, property_data, &|| PackedByteArray::default().to_variant()),
            VariantType::PACKED_INT32_ARRAY => Self::init_property_data_raw::<PackedInt32Array>(data, value, property_data, &|| PackedInt32Array::default().to_variant()),
            VariantType::PACKED_INT64_ARRAY => Self::init_property_data_raw::<PackedInt64Array>(data, value, property_data, &|| PackedInt64Array::default().to_variant()),
            VariantType::PACKED_FLOAT32_ARRAY => Self::init_property_data_raw::<PackedFloat32Array>(data, value, property_data, &|| PackedFloat32Array::default().to_variant()),
            VariantType::PACKED_FLOAT64_ARRAY => Self::init_property_data_raw::<PackedFloat64Array>(data, value, property_data, &|| PackedFloat64Array::default().to_variant()),
            VariantType::PACKED_STRING_ARRAY => Self::init_property_data_raw::<PackedStringArray>(data, value, property_data, &|| PackedStringArray::default().to_variant()),
            VariantType::PACKED_VECTOR2_ARRAY => Self::init_property_data_raw::<PackedVector2Array>(data, value, property_data, &|| PackedVector2Array::default().to_variant()),
            VariantType::PACKED_VECTOR3_ARRAY => Self::init_property_data_raw::<PackedVector3Array>(data, value, property_data, &|| PackedVector3Array::default().to_variant()),
            VariantType::PACKED_COLOR_ARRAY => Self::init_property_data_raw::<PackedColorArray>(data, value, property_data, &|| PackedColorArray::default().to_variant()),
            _ => unreachable!(),
        }
    }
    
    fn init_property_data_raw<T: FromGodot>(
        data: NonNull<u8>,
        value: Variant,
        property_data: &ComponetProperty,
        default: &dyn Fn() -> Variant,
    ) {
         let default_value = if value != Variant::nil() {
            value
        } else {
            (default)()
        };
        unsafe {
            let param_ptr: *mut u8 = &mut *data.as_ptr()
                .add(property_data.offset);
            let param_slice = std::slice
                ::from_raw_parts_mut(param_ptr, size_of::<T>());
            let value_ptr: *const ManuallyDrop<T> = &ManuallyDrop::new(
                default_value.to::<T>()
            );
            let value_slice = std::slice::from_raw_parts(
                value_ptr.cast::<u8>(),
                size_of::<T>(),
            );
            param_slice.copy_from_slice(value_slice);
        }
    }

    fn init_property_data_raw_variant(
        data: NonNull<u8>,
        value: Variant,
        property_data: &ComponetProperty,
    ) {
        let default_value = if value != Variant::nil() {
            value
        } else {
            Variant::default()
        };
        unsafe {
            let param_ptr:*mut u8 = &mut *data.as_ptr()
                .add(property_data.offset);
            let param_slice = std::slice
                ::from_raw_parts_mut(param_ptr, size_of::<Variant>());
            let value_ptr:*const ManuallyDrop<Variant> = &ManuallyDrop::new(
                default_value
            );
            let value_slice = std::slice::from_raw_parts(
                value_ptr.cast::<u8>(),
                size_of::<Variant>(),
            );
            param_slice.copy_from_slice(value_slice);
        }
    }
    
    // --- Deinitialization ---

    pub(crate) fn _deinitialize_property(
        data: NonNull<u8>,
        description: &ComponetDefinition,
        property: StringName,
    ) -> bool {
        let Some(property_data) = description
            .get_property(&property) else {
                show_error!(
                    "Property deinitialization failed",
                    "Can't deinit {} in {{component}}. Component has no property with that name",
                    property,
                );
                // return false;
            };

        Self::deinit_property_data(data, property_data);

        return true;
    }

    /// Deinitializes all properties in the data of the component.
    pub(crate) fn deinit_component_data(
        comp_data: NonNull<u8>,
        comp_def: &ComponetDefinition,
    ) {
        for p in comp_def.parameters.iter() {
            Self::deinit_property_data(comp_data, p);
        }
    }

    pub(crate) fn deinit_property_data(
        comp_data: NonNull<u8>,
        property_data: &ComponetProperty,
    ) {
        match property_data.gd_type_id {
            VariantType::NIL => panic!("Can't deinit \"Nil\" type in component"),
            VariantType::BOOL => Self::deinit_property_data_raw::<bool>(comp_data, property_data),
            VariantType::INT => Self::deinit_property_data_raw::<Int>(comp_data, property_data),
            VariantType::FLOAT => Self::deinit_property_data_raw::<Float>(comp_data, property_data),
            VariantType::STRING => Self::deinit_property_data_raw::<GString>(comp_data, property_data),
            VariantType::VECTOR2 => Self::deinit_property_data_raw::<Vector2>(comp_data, property_data),
            VariantType::VECTOR2I => Self::deinit_property_data_raw::<Vector2i>(comp_data, property_data),
            VariantType::RECT2 => Self::deinit_property_data_raw::<Rect2>(comp_data, property_data),
            VariantType::RECT2I => Self::deinit_property_data_raw::<Rect2i>(comp_data, property_data),
            VariantType::VECTOR3 => Self::deinit_property_data_raw::<Vector3>(comp_data, property_data),
            VariantType::VECTOR3I => Self::deinit_property_data_raw::<Vector3i>(comp_data, property_data),
            VariantType::TRANSFORM2D => Self::deinit_property_data_raw::<Transform2D>(comp_data, property_data),
            VariantType::VECTOR4 => Self::deinit_property_data_raw::<Vector4>(comp_data, property_data),
            VariantType::VECTOR4I => Self::deinit_property_data_raw::<Vector4i>(comp_data, property_data),
            VariantType::PLANE => Self::deinit_property_data_raw::<Plane>(comp_data, property_data),
            VariantType::QUATERNION => Self::deinit_property_data_raw::<Quaternion>(comp_data, property_data),
            VariantType::AABB => Self::deinit_property_data_raw::<Aabb>(comp_data, property_data),
            VariantType::BASIS => Self::deinit_property_data_raw::<Basis>(comp_data, property_data),
            VariantType::TRANSFORM3D => Self::deinit_property_data_raw::<Transform3D>(comp_data, property_data),
            VariantType::PROJECTION => Self::deinit_property_data_raw::<Projection>(comp_data, property_data),
            VariantType::COLOR => Self::deinit_property_data_raw::<Color>(comp_data, property_data),
            VariantType::STRING_NAME => Self::deinit_property_data_raw::<StringName>(comp_data, property_data),
            VariantType::NODE_PATH => Self::deinit_property_data_raw::<NodePath>(comp_data, property_data),
            VariantType::RID => Self::deinit_property_data_raw::<Rid>(comp_data, property_data),
            VariantType::OBJECT => Self::deinit_property_data_raw_variant(comp_data,property_data),
            VariantType::CALLABLE => Self::deinit_property_data_raw::<Callable>(comp_data, property_data),
            VariantType::SIGNAL => Self::deinit_property_data_raw::<Signal>(comp_data, property_data),
            VariantType::DICTIONARY => Self::deinit_property_data_raw_variant(comp_data,property_data),
            VariantType::ARRAY => Self::deinit_property_data_raw_variant(comp_data,property_data),
            VariantType::PACKED_BYTE_ARRAY => Self::deinit_property_data_raw::<PackedByteArray>(comp_data, property_data),
            VariantType::PACKED_INT32_ARRAY => Self::deinit_property_data_raw::<PackedInt32Array>(comp_data, property_data),
            VariantType::PACKED_INT64_ARRAY => Self::deinit_property_data_raw::<PackedInt64Array>(comp_data, property_data),
            VariantType::PACKED_FLOAT32_ARRAY => Self::deinit_property_data_raw::<PackedFloat32Array>(comp_data, property_data),
            VariantType::PACKED_FLOAT64_ARRAY => Self::deinit_property_data_raw::<PackedFloat64Array>(comp_data, property_data),
            VariantType::PACKED_STRING_ARRAY => Self::deinit_property_data_raw::<PackedStringArray>(comp_data, property_data),
            VariantType::PACKED_VECTOR2_ARRAY => Self::deinit_property_data_raw::<PackedVector2Array>(comp_data, property_data),
            VariantType::PACKED_VECTOR3_ARRAY => Self::deinit_property_data_raw::<PackedVector3Array>(comp_data, property_data),
            VariantType::PACKED_COLOR_ARRAY => Self::deinit_property_data_raw::<PackedColorArray>(comp_data, property_data),
            _ => unreachable!(),
        }
    }

    fn deinit_property_data_raw<T> (
        comp_data: NonNull<u8>,
        property_data: &ComponetProperty,
    ) {
        let property = unsafe {
            comp_data.as_ptr()
                .add(property_data.offset)
                .cast::<ManuallyDrop<T>>()
                .as_mut()
                .unwrap()
        };

        drop(unsafe { ManuallyDrop::take(property) })
    }

    fn deinit_property_data_raw_variant(
        comp_data: NonNull<u8>,
        property_data: &ComponetProperty,
    ) {
        let property = unsafe {
            comp_data.as_ptr()
                .add(property_data.offset)
                .cast::<ManuallyDrop<Variant>>()
                .as_mut()
                .unwrap()
        };
        
        drop(unsafe { ManuallyDrop::take(property) })
    }

    fn get_data(&self) -> NonNull<u8> {
        unsafe { NonNull::new_unchecked(flecs::ecs_get_mut_id(
            self.world.bind().raw(),
            self.entity_id,
            self.get_flecs_id(),
        ).cast::<u8>()) }
    }

    /// Returns the Flecs ID of this component's type.
    pub(crate) fn get_flecs_id(&self) -> EntityId {
        self.component_id
    }

    // --- Hooks ---

    pub(crate) fn set_hooks_in_component(world: &_GlecsBaseWorld, componnet: EntityId) {
        let world_ptr = world.raw();
        unsafe { flecs::ecs_set_hooks_id(
            world_ptr,
            componnet,
            &flecs::ecs_type_hooks_t {
                ctor: Some(Self::ctor_hook),
                dtor: Some(Self::dtor_hook),
                move_: Some(Self::move_hook),
                ctor_move_dtor: Some(Self::ctor_move_dtor_hook),
                binding_ctx: HookContext::new(world.to_gd(), componnet)
                    .to_leaked() as *mut c_void,
                binding_ctx_free: Some(HookContext::binding_ctx_free),
                ..Default::default()
            },
        ) };
    }

    pub(crate) extern "C" fn ctor_hook(
        ptr: *mut c_void,
		count: i32,
		type_info: *const flecs::ecs_type_info_t,
    ) {
        let count = count as usize;
        let hook_context = HookContext::ref_leaked(
            unsafe { &*type_info }.hooks.binding_ctx
        );
        let comp_desc = hook_context.world.bind()
            .get_component_description(hook_context.component_id)
            .unwrap();

        for i in 0..count {
            let counted_ptr = unsafe {
                ptr.add(i * comp_desc.layout.size())
            };

            // Write sane defaults to data
            let data = unsafe {
                NonNull::new_unchecked(counted_ptr as *mut u8)
            };
            _GlecsBaseComponent::init_component_data(
                data,
                &comp_desc,
            );
        }
    }

    pub(crate) extern "C" fn dtor_hook(
        ptr: *mut c_void,
		count: i32,
		type_info: *const flecs::ecs_type_info_t,
    ) {
        let count = count as usize;
        let hook_context = HookContext::ref_leaked(
            unsafe { &*type_info }.hooks.binding_ctx
        );
        let comp_desc = hook_context.world.bind()
            .get_component_description(hook_context.component_id)
            .unwrap();

        for i in 0..count {
            let counted_ptr = unsafe {
                ptr.add(i * comp_desc.layout.size())
            };

            // Call destructor for each property
            let data = unsafe {
                NonNull::new_unchecked(counted_ptr as *mut u8)
            };
            _GlecsBaseComponent::deinit_component_data(
                data,
                &comp_desc,
            );
        }
    }

    pub(crate) extern "C" fn move_hook(
		dst_ptr: *mut c_void,
		src_ptr: *mut c_void,
		count: i32,
		type_info: *const flecs::ecs_type_info_t,
	) {
        let count = count as usize;
        let hook_context = HookContext::ref_leaked(
            unsafe { &*type_info }.hooks.binding_ctx
        );
        let comp_desc = hook_context.world.bind()
            .get_component_description(hook_context.component_id)
            .unwrap();

        for i in 0..count {
            let src = unsafe {
                std::slice::from_raw_parts_mut(
                    src_ptr.add(i * comp_desc.layout.size())
                        as *mut u8,
                    comp_desc.layout.size(),
                )
            };
            let dst = unsafe {
                std::slice::from_raw_parts_mut(
                    dst_ptr.add(i * comp_desc.layout.size())
                        as *mut u8,
                    comp_desc.layout.size(),
                )
            };

            // Move contents
            dst.copy_from_slice(src);

            // Reset src so that the destructor does not attempt to deinit
            // the moved data
            _GlecsBaseComponent::init_component_data(
                unsafe { NonNull::new_unchecked(src.as_mut_ptr()) },
                &comp_desc,
            );
        }
    }

    pub(crate) extern "C" fn ctor_move_dtor_hook(
		dst_ptr: *mut c_void,
		src_ptr: *mut c_void,
		count: i32,
		type_info: *const flecs::ecs_type_info_t,
	) {
        let count = count as usize;
        let hook_context = HookContext::ref_leaked(
            unsafe { &*type_info }.hooks.binding_ctx
        );
        let comp_desc = hook_context.world.bind()
            .get_component_description(hook_context.component_id)
            .unwrap();

        for i in 0..count {
            let src = unsafe {
                std::slice::from_raw_parts_mut(
                    src_ptr.add(i * comp_desc.layout.size())
                        as *mut u8,
                    comp_desc.layout.size(),
                )
            };
            let dst = unsafe {
                std::slice::from_raw_parts_mut(
                    dst_ptr.add(i * comp_desc.layout.size())
                        as *mut u8,
                    comp_desc.layout.size(),
                )
            };

            // Move contents
            dst.copy_from_slice(src);
        }
    }
}

impl EntityLike for _GlecsBaseComponent {
    fn get_world(&self) -> Gd<_GlecsBaseWorld> {
        self.world.clone()
    }

    fn get_flecs_id(&self) -> EntityId {
        self.component_id
    }

    fn delete(&self) {
        unsafe { flecs::ecs_remove_id(
            self.world.bind().raw(),
            self.entity_id,
            self.get_flecs_id(),
        ) };
    }

    fn is_valid(&self) -> bool{
        // Check world
        let Some(_) = self.world.is_instance_valid()
            .then_some(())
            else { return false };

        // Check master entity
        let Some(_) = _GlecsBindings::id_is_alive(self.world.clone(), self.entity_id)
            .then_some(())
            else { return false };

        // Check component type is alive
        match self.get_flecs_id() {
            c if
                _GlecsBindings::id_is_pair(c)
                && _GlecsBindings::has_id(
                    self.world.clone(),
                    _GlecsBindings::pair_first(c),
                    unsafe { flecs::FLECS_IDEcsComponentID_ },
                )
            => {
                // ID is a pair, and the first part is a component
                let id = _GlecsBindings::pair_first(c);
                let Some(_) = _GlecsBindings::id_is_alive(self.world.clone(), id)
                    .then_some(())
                    else { return false };
            },

            c if
                _GlecsBindings::id_is_pair(c)
                && _GlecsBindings::has_id(
                    self.world.clone(),
                    _GlecsBindings::pair_second(c),
                    unsafe { flecs::FLECS_IDEcsComponentID_ },
                )
            => {
                // ID is a pair, and the second part is a component
                let id = _GlecsBindings::pair_second(c);
                let Some(_) = _GlecsBindings::id_is_alive(self.world.clone(), id)
                    .then_some(())
                    else { return false };
            },

            c => {
                // ID is a normal component
                let Some(_) = _GlecsBindings::id_is_alive(self.world.clone(), c)
                    .then_some(())
                    else { return false };
            },

        }

        // Check that the entity has this component attached
        let ett_id = self.entity_id;
        let comp_id = self.get_flecs_id();
        let Some(_) = _GlecsBindings::has_id(self.world.clone(), ett_id, comp_id)
            .then_some(())
            else { return false };

        return true;
    }

    fn validate(&self) {
        // Check world
        self.world.is_instance_valid()
            .then_some(())
            .expect("Component's world was deleted");

        // Check master entity
        _GlecsBindings::id_is_alive(self.world.clone(), self.entity_id)
            .then_some(())
            .expect("The entity this component was attached to was delted.");

        // Check component type is alive
        match self.get_flecs_id() {
            c if
                _GlecsBindings::id_is_pair(c)
                && _GlecsBindings::has_id(
                    self.world.clone(),
                    _GlecsBindings::pair_first(c),
                    unsafe { flecs::FLECS_IDEcsComponentID_ },
                )
            => {
                // ID is a pair, and the first part is a component
                let id = _GlecsBindings::pair_first(c);
                _GlecsBindings::id_is_alive(self.world.clone(), id)
                    .then_some(())
                    .expect("Component type was deleted.");
            },

            c if
                _GlecsBindings::id_is_pair(c)
                && _GlecsBindings::has_id(
                    self.world.clone(),
                    _GlecsBindings::pair_second(c),
                    unsafe { flecs::FLECS_IDEcsComponentID_ },
                )
            => {
                // ID is a pair, and the second part is a component
                let id = _GlecsBindings::pair_second(c);
                _GlecsBindings::id_is_alive(self.world.clone(), id)
                    .then_some(())
                    .expect("Component type was deleted.");
            },

            c => {
                // ID is a normal component
                _GlecsBindings::id_is_alive(self.world.clone(), c)
                    .then_some(())
                    .expect("Component type was deleted.");
            },

        }

        // Check that the entity has this component attached
        let ett_id = self.entity_id;
        let comp_id = self.get_flecs_id();
        _GlecsBindings::has_id(self.world.clone(), ett_id, comp_id)
            .then_some(())
            .expect(&format!(
                "Component was removed from its entity. Component ID: {}, Entity ID: {}",
                comp_id,
                ett_id,
            ));
    }
}

#[godot_api]
impl IRefCounted for _GlecsBaseComponent {
    fn get_property(&self, property: StringName) -> Option<Variant> {
        Some(self._get_property(property))
    }

    fn set_property(&mut self, property: StringName, v:Variant) -> bool{
        self._set_property(property, v)
    }
}

impl std::fmt::Debug for _GlecsBaseComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("_GlecsComponent")
            .field("base", &self.base)
            .field("component_definition", &self.component_definition)
            .field("world", &self.world)
            .finish()
    }
}

pub(crate) struct HookContext {
    component_id: EntityId,
    world: Gd<_GlecsBaseWorld>,
} impl HookContext {
    pub(crate) fn new(world: Gd<_GlecsBaseWorld>, component_id: EntityId) -> Self {
        Self {
            world,
            component_id
        }
    }

    fn ref_leaked<'a>(from: *mut c_void) -> &'a mut Self {
        unsafe { (from as *mut Self).as_mut().unwrap() }
    }

    unsafe fn take_leaked(from: *mut c_void) -> Box<Self> {
        unsafe { Box::from_raw(from as *mut Self) }
    }

    pub(crate) fn to_leaked(self) -> *mut Self {
        let ptr:*mut Self = Box::leak(Box::new(self));
        ptr
    }

    pub(crate) extern "C" fn binding_ctx_free(ctx: *mut c_void) {
        drop(unsafe { Self::take_leaked(ctx ) } )
    }
}