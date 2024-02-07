
use std::fmt::Debug;
use std::rc::Rc;
use std::mem::size_of;

use flecs::EntityId;
use godot::engine::notify::ObjectNotification;
use godot::prelude::*;

use crate::component_definitions::ComponetDefinition;
use crate::component_definitions::ComponetProperty;
use crate::entity::FREED_BY_ENTITY_TAG;
use crate::show_error;

/// An ECS component.
#[derive(GodotClass)]
#[class(base=Object)]
pub struct _BaseGEComponent {
    #[base] pub(crate) base: Base<Object>,
    /// The Flecs ID for the type of this component.
    pub(crate) flecs_id: EntityId,
    pub(crate) data: *mut [u8],
    pub(crate) component_definition: Rc<ComponetDefinition>,
}
#[godot_api]
impl _BaseGEComponent {

    /// Returns the name of the the type of this component.
    #[func]
    fn get_component_type_name(&self) -> StringName {
        self.component_definition.name.clone()
    }

    /// Returns a property from the component data.
    #[func]
    fn getc(&self, property: StringName) -> Variant {
        self._get_property(property)
    }

    /// Sets a property in the component data.
    #[func]
    fn setc(&mut self, property: StringName, value:Variant) {
        self._set_property(property.clone(), value.clone());
    }

    /// Prevent user from freeing a component.
    #[func]
    fn free(&self) {
        return;
    }

    pub(crate) fn _get_property(
		&self,
		property:StringName,
	) -> Variant {
        let Some(property_data) = self
            .component_definition
            .parameters
            .get(&property)
            else {
                show_error!(
                    "Failed to get property",
                    "No property named \"{}\" in component of type \"{}\"",
                    property,
                    self.get_component_type_name(),
                );
                return Variant::nil();
            };
        
        fn get_param<T: ToGodot + Clone + Debug>(
            data:*mut [u8],
            property_data: &ComponetProperty
        ) -> Variant {
            unsafe {
                let param:*mut u8 = &mut (*data)[property_data.offset];
                let value = param as *mut T;
                let copied = (*value).clone();
                let variant = Variant::from(copied);
                return variant;
            }
        }
        
        let value =  match property_data.gd_type_id {
            VariantType::Bool => get_param::<bool>(self.data, property_data),
            VariantType::Int => get_param::<i32>(self.data, property_data),
            VariantType::Float => get_param::<f32>(self.data, property_data),
            VariantType::String => get_param::<GString>(self.data, property_data),
            VariantType::Vector2 => get_param::<Vector2>(self.data, property_data),
            VariantType::Vector2i => get_param::<Vector2i>(self.data, property_data),
            VariantType::Rect2 => get_param::<Rect2>(self.data, property_data),
            VariantType::Rect2i => get_param::<Rect2i>(self.data, property_data),
            VariantType::Vector3 => get_param::<Vector3>(self.data, property_data),
            VariantType::Vector3i => get_param::<Vector3i>(self.data, property_data),
            VariantType::Transform2D => get_param::<Transform2D>(self.data, property_data),
            VariantType::Vector4 => get_param::<Vector4>(self.data, property_data),
            VariantType::Vector4i => get_param::<Vector4i>(self.data, property_data),
            VariantType::Plane => get_param::<Plane>(self.data, property_data),
            VariantType::Quaternion => get_param::<Quaternion>(self.data, property_data),
            VariantType::Aabb => get_param::<Aabb>(self.data, property_data),
            VariantType::Basis => get_param::<Basis>(self.data, property_data),
            VariantType::Transform3D => get_param::<Transform3D>(self.data, property_data),
            VariantType::Projection => get_param::<Projection>(self.data, property_data),
            VariantType::Color => get_param::<Color>(self.data, property_data),
            VariantType::StringName => get_param::<StringName>(self.data, property_data),
            VariantType::NodePath => get_param::<NodePath>(self.data, property_data),
            VariantType::Rid => get_param::<Rid>(self.data, property_data),
            VariantType::Object => todo!("Object doesn't support conversion to variant or copying"), /* get_param::<Object>(self.data, property_data), */
            VariantType::Callable => get_param::<Callable>(self.data, property_data),
            VariantType::Signal => get_param::<Signal>(self.data, property_data),
            VariantType::Dictionary => todo!("Reading causes crashes"), /* get_param::<Dictionary>(self.data, property_data), */
            VariantType::Array => todo!("Reading causes crashes"), /* get_param::<Array<Variant>>(self.data, property_data), */
            VariantType::PackedByteArray => get_param::<PackedByteArray>(self.data, property_data),
            VariantType::PackedInt32Array => get_param::<PackedInt32Array>(self.data, property_data),
            VariantType::PackedInt64Array => get_param::<PackedInt64Array>(self.data, property_data),
            VariantType::PackedFloat32Array => get_param::<PackedFloat32Array>(self.data, property_data),
            VariantType::PackedFloat64Array => get_param::<PackedFloat64Array>(self.data, property_data),
            VariantType::PackedStringArray => get_param::<PackedStringArray>(self.data, property_data),
            VariantType::PackedVector2Array => get_param::<PackedVector2Array>(self.data, property_data),
            VariantType::PackedVector3Array => get_param::<PackedVector3Array>(self.data, property_data),
            VariantType::PackedColorArray => get_param::<PackedColorArray>(self.data, property_data),

            _ => todo!(),
        };
        value
    }

    pub(crate) fn _set_property(
		&mut self,
		property:StringName,
		value:Variant,
	) -> bool {
        let Some(property_data) = self
            .component_definition
            .parameters
            .get(&property) else {
                show_error!(
                    "Failed to set property",
                    "No property named \"{}\" in component of type \"{}\"",
                    property,
                    self.get_component_type_name(),
                );
                return false;
            };

        if value.get_type() != property_data.gd_type_id {
            show_error!(
                "Failed to set property",
                "Expected type {:?}, but got type {:?}.",
                property_data.gd_type_id,
                value.get_type(),
            );
            return true;
        }
        
        fn set_param<T: FromGodot + ToGodot + Debug + Clone>(
            data:*mut [u8],
            value: Variant,
            property_data: &ComponetProperty,
        ) {
            unsafe {
                let param_ptr:*mut u8 = &mut (*data)[property_data.offset];
                *(param_ptr as *mut T) = value.to::<T>().clone();
            }
        }
        
        match property_data.gd_type_id {
            VariantType::Bool => set_param::<bool>(self.data, value, property_data),
            VariantType::Int => set_param::<i32>(self.data, value, property_data),
            VariantType::Float => set_param::<f32>(self.data, value, property_data),
            VariantType::String => set_param::<GString>(self.data, value, property_data),
            VariantType::Vector2 => set_param::<Vector2>(self.data, value, property_data),
            VariantType::Vector2i => set_param::<Vector2i>(self.data, value, property_data),
            VariantType::Rect2 => set_param::<Rect2>(self.data, value, property_data),
            VariantType::Rect2i => set_param::<Rect2i>(self.data, value, property_data),
            VariantType::Vector3 => set_param::<Vector3>(self.data, value, property_data),
            VariantType::Vector3i => set_param::<Vector3i>(self.data, value, property_data),
            VariantType::Transform2D => set_param::<Transform2D>(self.data, value, property_data),
            VariantType::Vector4 => set_param::<Vector4>(self.data, value, property_data),
            VariantType::Vector4i => set_param::<Vector4i>(self.data, value, property_data),
            VariantType::Plane => set_param::<Plane>(self.data, value, property_data),
            VariantType::Quaternion => set_param::<Quaternion>(self.data, value, property_data),
            VariantType::Aabb => set_param::<Aabb>(self.data, value, property_data),
            VariantType::Basis => set_param::<Basis>(self.data, value, property_data),
            VariantType::Transform3D => set_param::<Transform3D>(self.data, value, property_data),
            VariantType::Projection => set_param::<Projection>(self.data, value, property_data),
            VariantType::Color => set_param::<Color>(self.data, value, property_data),
            VariantType::StringName => set_param::<StringName>(self.data, value, property_data),
            VariantType::NodePath => set_param::<NodePath>(self.data, value, property_data),
            VariantType::Rid => set_param::<Rid>(self.data, value, property_data),
            VariantType::Object => todo!("Object doesn't support conversion to variant or copying"), /* set_param::<Object>(self.data, value, property_data), */
            VariantType::Callable => set_param::<Callable>(self.data, value, property_data),
            VariantType::Signal => set_param::<Signal>(self.data, value, property_data),
            VariantType::Dictionary => set_param::<Dictionary>(self.data, value, property_data),
            VariantType::Array => set_param::<Array<Variant>>(self.data, value, property_data),
            VariantType::PackedByteArray => set_param::<PackedByteArray>(self.data, value, property_data),
            VariantType::PackedInt32Array => set_param::<PackedInt32Array>(self.data, value, property_data),
            VariantType::PackedInt64Array => set_param::<PackedInt64Array>(self.data, value, property_data),
            VariantType::PackedFloat32Array => set_param::<PackedFloat32Array>(self.data, value, property_data),
            VariantType::PackedFloat64Array => set_param::<PackedFloat64Array>(self.data, value, property_data),
            VariantType::PackedStringArray => set_param::<PackedStringArray>(self.data, value, property_data),
            VariantType::PackedVector2Array => set_param::<PackedVector2Array>(self.data, value, property_data),
            VariantType::PackedVector3Array => set_param::<PackedVector3Array>(self.data, value, property_data),
            VariantType::PackedColorArray => set_param::<PackedColorArray>(self.data, value, property_data),

            _ => todo!(),
        }

        return true;
    }

    // Similar to [_set_property], except it does not call the destructor.
    pub(crate) fn _initialize_property(
        data:&mut [u8],
        description:&ComponetDefinition,
        property:StringName,
        value:Variant, // TODO: Utilize the initialization value
    ) -> bool {
        let Some(property_data) = description
            .parameters
            .get(&property) else {
                show_error!(
                    "Property initialization failed",
                    "Can't write to {} in {{component}}. Component has no property with that name",
                    property,
                );
                return false;
            };
        
        fn init_param<T: FromGodot + ToGodot + Debug + Default + Clone>(
            data:*mut [u8],
            value: Variant,
            property_data: &ComponetProperty,
        ) {
             let default_value = if value != Variant::nil() {
                value
            } else {
                Variant::from(T::default())
            };
            unsafe {
                let param_ptr:*mut u8 = &mut (*data)[property_data.offset];
                let param_slice = std::slice::from_raw_parts_mut(param_ptr, size_of::<T>());
                let value_ptr:*const T = &default_value.to::<T>().clone();
                let value_slice = std::slice::from_raw_parts(value_ptr as *const u8, size_of::<T>());
                param_slice.copy_from_slice(value_slice);
            }
        }
        
        match property_data.gd_type_id {
            VariantType::Bool => init_param::<bool>(data, value, property_data),
            VariantType::Int => init_param::<i32>(data, value, property_data),
            VariantType::Float => init_param::<f32>(data, value, property_data),
            VariantType::String => init_param::<GString>(data, value, property_data),
            VariantType::Vector2 => init_param::<Vector2>(data, value, property_data),
            VariantType::Vector2i => init_param::<Vector2i>(data, value, property_data),
            VariantType::Rect2 => init_param::<Rect2>(data, value, property_data),
            VariantType::Rect2i => init_param::<Rect2i>(data, value, property_data),
            VariantType::Vector3 => init_param::<Vector3>(data, value, property_data),
            VariantType::Vector3i => init_param::<Vector3i>(data, value, property_data),
            VariantType::Transform2D => init_param::<Transform2D>(data, value, property_data),
            VariantType::Vector4 => init_param::<Vector4>(data, value, property_data),
            VariantType::Vector4i => init_param::<Vector4i>(data, value, property_data),
            VariantType::Plane => todo!("Can't initialize planes with a sane default"),
            VariantType::Quaternion => init_param::<Quaternion>(data, value, property_data),
            VariantType::Aabb => init_param::<Aabb>(data, value, property_data),
            VariantType::Basis => init_param::<Basis>(data, value, property_data),
            VariantType::Transform3D => init_param::<Transform3D>(data, value, property_data),
            VariantType::Projection => init_param::<Projection>(data, value, property_data),
            VariantType::Color => init_param::<Color>(data, value, property_data),
            VariantType::StringName => init_param::<StringName>(data, value, property_data),
            VariantType::NodePath => init_param::<NodePath>(data, value, property_data),
            VariantType::Rid => todo!("Can't initialize RIDs with a sane default"),
            VariantType::Object => todo!("Objects don't support conversion to variant or copying"), /* get_param::<Object>(data, property_data), */
            VariantType::Callable => todo!("Can't initialize callables with a sane default"),
            VariantType::Signal => todo!("Can't initialize signals with a sane default"),
            VariantType::Dictionary => init_param::<Dictionary>(data, value, property_data),
            VariantType::Array => init_param::<Array<Variant>>(data, value, property_data),
            VariantType::PackedByteArray => init_param::<PackedByteArray>(data, value, property_data),
            VariantType::PackedInt32Array => init_param::<PackedInt32Array>(data, value, property_data),
            VariantType::PackedInt64Array => init_param::<PackedInt64Array>(data, value, property_data),
            VariantType::PackedFloat32Array => init_param::<PackedFloat32Array>(data, value, property_data),
            VariantType::PackedFloat64Array => init_param::<PackedFloat64Array>(data, value, property_data),
            VariantType::PackedStringArray => init_param::<PackedStringArray>(data, value, property_data),
            VariantType::PackedVector2Array => init_param::<PackedVector2Array>(data, value, property_data),
            VariantType::PackedVector3Array => init_param::<PackedVector3Array>(data, value, property_data),
            VariantType::PackedColorArray => init_param::<PackedColorArray>(data, value, property_data),

            _ => todo!(),
        }

        return true;
    }

    fn on_free(&mut self) {
        let mut base = self.base_mut();
        if !base.has_meta(FREED_BY_ENTITY_TAG.into()) {
            base.cancel_free();
            return;
        }
    }
}
#[godot_api]
impl IObject for _BaseGEComponent {
    fn on_notification(&mut self, what: ObjectNotification) {
        match what {
            ObjectNotification::Predelete => {
                self.on_free()
            },
            _ => {},
        }
    }
    
    // fn get_property(&self, property: StringName) -> Option<Variant> {
    //     Some(self._get_property(property))
    // }

    // fn set_property(&mut self, property: StringName, v:Variant) -> bool{
    //     self._set_property(property, v)
    // }
}