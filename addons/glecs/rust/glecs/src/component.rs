
use std::fmt::Debug;
use std::ptr::NonNull;
use std::rc::Rc;
use std::mem::size_of;

use flecs::EntityId;
use godot::engine::notify::ObjectNotification;
use godot::prelude::*;

use crate::component_definitions::ComponetDefinition;
use crate::component_definitions::ComponetProperty;
use crate::entity::FREED_BY_ENTITY_TAG;
use crate::show_error;
use crate::world::_BaseGEWorld;

/// An ECS component.
#[derive(GodotClass)]
#[class(base=Object, no_init)]
pub struct _BaseGEComponent {
    pub(crate) base: Base<Object>,
    pub(crate) component_definition: Rc<ComponetDefinition>,
    pub(crate) world: Gd<_BaseGEWorld>,
    pub(crate) get_data_fn_ptr: Box<dyn Fn(&Self) -> NonNull<[u8]>>,
}
#[godot_api]
impl _BaseGEComponent {
    /// Copies the data from the given component to this one.
    #[func]
    fn copy_from_component(&mut self, from_component:Gd<_BaseGEComponent>) {
        if self.get_flecs_id() != from_component.bind().get_flecs_id() {
            show_error!(
                "Failed to copy component",
                "Destination component is of type {}, but source component is of type {}",
                self.base().get_script(),
                from_component.bind().base().get_script(),
            )
        }
        unsafe {
            self.get_data().as_mut().copy_from_slice(
                from_component.bind().get_data().as_ref(),
            );
        }
    }

    /// Returns the name of the the type of this component.
    #[func]
    fn get_component_type_name(&self) -> StringName {
        self.component_definition.name.clone()
    }

    /// Returns a property from the component data.
    #[func]
    fn getc(&self, property: StringName) -> Variant {
        let v = self._get_property(property.clone());
        v
    }

    /// Sets a property in the component data.
    #[func]
    fn setc(&mut self, property: StringName, value:Variant) {
        if !self._set_property(property.clone(), value.clone()) {
            show_error!(
                "Failed to set property",
                "No property named \"{}\" in component of type \"{}\"",
                property,
                self.get_component_type_name(),
            );
        }
    }

    /// Prevent user from freeing a component.
    #[func]
    fn free(&self) {
        return;
    }

    pub(crate) fn create_initial_data(def: &ComponetDefinition, parameters:Variant) -> Box<[u8]> {
        let mut data = Vec::<u8>::new();
        data.resize(def.layout.size(), 0);
        
        match parameters.get_type() {
            VariantType::Array => {
                let parameters = parameters.to::<VariantArray>();
                for
                    (i, property_meta)
                in def.parameters.iter().enumerate() {
                    let prop_value = if i < parameters.len() {
                        // Get value from passed parameters
                        let parameter = parameters.get(i);
                        let value = if
                            parameter.get_type() == property_meta.gd_type_id
                        {
                            parameter
                        } else {
                            // Parameter is wrong type, get value
                            // from component's default
                            def.get_property_default_value(
                                &property_meta.name.to_string(),
                            )
                        };
                        value
                    } else {
                        // Get value from component's default
                        def.get_property_default_value(
                            &property_meta.name.to_string(),
                        )
                    };

                    Self::init_data_property(&mut data, prop_value, &property_meta)
                }
            },
            VariantType::Nil => {
                for property_meta in def.parameters.iter() {
                    let default = def.get_property_default_value(
                        &property_meta.name.to_string(),
                    );
                    Self::init_data_property(&mut data, default, &property_meta)
                }
            },
            _ => todo!(),
        }
    
        data.into_boxed_slice()
    }
    
    pub(crate) fn get_data_property(
        data: &mut [u8],
        property_data: &ComponetProperty,
    ) -> Variant{
        match property_data.gd_type_id {
            VariantType::Nil => panic!("Can't set \"Nil\" type in component"),
            VariantType::Bool => Self::get_data_property_raw::<bool>(data, property_data.offset).to_variant(),
            VariantType::Int => Self::get_data_property_raw::<i32>(data, property_data.offset).to_variant(),
            VariantType::Float => Self::get_data_property_raw::<f32>(data, property_data.offset).to_variant(),
            VariantType::String => Self::get_data_property_raw::<GString>(data, property_data.offset).to_variant(),
            VariantType::Vector2 => Self::get_data_property_raw::<Vector2>(data, property_data.offset).to_variant(),
            VariantType::Vector2i => Self::get_data_property_raw::<Vector2i>(data, property_data.offset).to_variant(),
            VariantType::Rect2 => Self::get_data_property_raw::<Rect2>(data, property_data.offset).to_variant(),
            VariantType::Rect2i => Self::get_data_property_raw::<Rect2i>(data, property_data.offset).to_variant(),
            VariantType::Vector3 => Self::get_data_property_raw::<Vector3>(data, property_data.offset).to_variant(),
            VariantType::Vector3i => Self::get_data_property_raw::<Vector3i>(data, property_data.offset).to_variant(),
            VariantType::Transform2D => Self::get_data_property_raw::<Transform2D>(data, property_data.offset).to_variant(),
            VariantType::Vector4 => Self::get_data_property_raw::<Vector4>(data, property_data.offset).to_variant(),
            VariantType::Vector4i => Self::get_data_property_raw::<Vector4i>(data, property_data.offset).to_variant(),
            VariantType::Plane => Self::get_data_property_raw::<Plane>(data, property_data.offset).to_variant(),
            VariantType::Quaternion => Self::get_data_property_raw::<Quaternion>(data, property_data.offset).to_variant(),
            VariantType::Aabb => Self::get_data_property_raw::<Aabb>(data, property_data.offset).to_variant(),
            VariantType::Basis => Self::get_data_property_raw::<Basis>(data, property_data.offset).to_variant(),
            VariantType::Transform3D => Self::get_data_property_raw::<Transform3D>(data, property_data.offset).to_variant(),
            VariantType::Projection => Self::get_data_property_raw::<Projection>(data, property_data.offset).to_variant(),
            VariantType::Color => Self::get_data_property_raw::<Color>(data, property_data.offset).to_variant(),
            VariantType::StringName => Self::get_data_property_raw::<StringName>(data, property_data.offset).to_variant(),
            VariantType::NodePath => Self::get_data_property_raw::<NodePath>(data, property_data.offset).to_variant(),
            VariantType::Rid => Self::get_data_property_raw::<Rid>(data, property_data.offset).to_variant(),
            VariantType::Object => Self::get_data_property_raw_variant(data, property_data.offset).to_variant(),
            VariantType::Callable => Self::get_data_property_raw::<Callable>(data, property_data.offset).to_variant(),
            VariantType::Signal => Self::get_data_property_raw::<Signal>(data, property_data.offset).to_variant(),
            VariantType::Dictionary => Self::get_data_property_raw_variant(data, property_data.offset).to_variant(),
            VariantType::Array => Self::get_data_property_raw_variant(data, property_data.offset).to_variant(),
            VariantType::PackedByteArray => Self::get_data_property_raw::<PackedByteArray>(data, property_data.offset).to_variant(),
            VariantType::PackedInt32Array => Self::get_data_property_raw::<PackedInt32Array>(data, property_data.offset).to_variant(),
            VariantType::PackedInt64Array => Self::get_data_property_raw::<PackedInt64Array>(data, property_data.offset).to_variant(),
            VariantType::PackedFloat32Array => Self::get_data_property_raw::<PackedFloat32Array>(data, property_data.offset).to_variant(),
            VariantType::PackedFloat64Array => Self::get_data_property_raw::<PackedFloat64Array>(data, property_data.offset).to_variant(),
            VariantType::PackedStringArray => Self::get_data_property_raw::<PackedStringArray>(data, property_data.offset).to_variant(),
            VariantType::PackedVector2Array => Self::get_data_property_raw::<PackedVector2Array>(data, property_data.offset).to_variant(),
            VariantType::PackedVector3Array => Self::get_data_property_raw::<PackedVector3Array>(data, property_data.offset).to_variant(),
            VariantType::PackedColorArray => Self::get_data_property_raw::<PackedColorArray>(data, property_data.offset).to_variant(),
        }
    }
    
    pub(crate) fn get_data_property_raw<T: ToGodot + Clone + Debug>(
        data: &mut [u8],
        offset: usize,
    ) -> T {
        let param = unsafe {
            NonNull::new_unchecked(&mut (data)[offset])
                .cast::<T>()
                .as_ref()
                .clone()
        };
        return param;
    }
    
    fn get_data_property_raw_variant(
        data: &mut [u8],
        offset: usize,
    ) -> Variant {
        let variant = unsafe { NonNull::new_unchecked(&mut (*data)[offset]) };
        let variant = variant.cast::<Variant>();
        let variant = unsafe { variant.as_ref() };
        let variant = variant.clone();
        return variant;
    }

    pub(crate) fn init_data_property(
        data: &mut [u8],
        value: Variant,
        property_data: &ComponetProperty,
    ) {
        match property_data.gd_type_id {
            VariantType::Nil => panic!("Can't init \"Nil\" type in component"),
            VariantType::Bool => Self::init_data_property_raw::<bool>(data, value, property_data, &|| bool::default().to_variant()),
            VariantType::Int => Self::init_data_property_raw::<i32>(data, value, property_data, &|| i32::default().to_variant()),
            VariantType::Float => Self::init_data_property_raw::<f32>(data, value, property_data, &|| f32::default().to_variant()),
            VariantType::String => Self::init_data_property_raw::<GString>(data, value, property_data, &|| GString::default().to_variant()),
            VariantType::Vector2 => Self::init_data_property_raw::<Vector2>(data, value, property_data, &|| Vector2::default().to_variant()),
            VariantType::Vector2i => Self::init_data_property_raw::<Vector2i>(data, value, property_data, &|| Vector2i::default().to_variant()),
            VariantType::Rect2 => Self::init_data_property_raw::<Rect2>(data, value, property_data, &|| Rect2::default().to_variant()),
            VariantType::Rect2i => Self::init_data_property_raw::<Rect2i>(data, value, property_data, &|| Rect2i::default().to_variant()),
            VariantType::Vector3 => Self::init_data_property_raw::<Vector3>(data, value, property_data, &|| Vector3::default().to_variant()),
            VariantType::Vector3i => Self::init_data_property_raw::<Vector3i>(data, value, property_data, &|| Vector3i::default().to_variant()),
            VariantType::Transform2D => Self::init_data_property_raw::<Transform2D>(data, value, property_data, &|| Transform2D::default().to_variant()),
            VariantType::Vector4 => Self::init_data_property_raw::<Vector4>(data, value, property_data, &|| Vector4::default().to_variant()),
            VariantType::Vector4i => Self::init_data_property_raw::<Vector4i>(data, value, property_data, &|| Vector4i::default().to_variant()),
            VariantType::Plane => Self::init_data_property_raw::<Plane>(data, value, property_data, &|| Plane::invalid().to_variant()),
            VariantType::Quaternion => Self::init_data_property_raw::<Quaternion>(data, value, property_data, &|| Quaternion::default().to_variant()),
            VariantType::Aabb => Self::init_data_property_raw::<Aabb>(data, value, property_data, &|| Aabb::default().to_variant()),
            VariantType::Basis => Self::init_data_property_raw::<Basis>(data, value, property_data, &|| Basis::default().to_variant()),
            VariantType::Transform3D => Self::init_data_property_raw::<Transform3D>(data, value, property_data, &|| Transform3D::default().to_variant()),
            VariantType::Projection => Self::init_data_property_raw::<Projection>(data, value, property_data, &|| Projection::default().to_variant()),
            VariantType::Color => Self::init_data_property_raw::<Color>(data, value, property_data, &|| Color::default().to_variant()),
            VariantType::StringName => Self::init_data_property_raw::<StringName>(data, value, property_data, &|| StringName::default().to_variant()),
            VariantType::NodePath => Self::init_data_property_raw::<NodePath>(data, value, property_data, &|| NodePath::default().to_variant()),
            VariantType::Rid => Self::init_data_property_raw::<Rid>(data, value, property_data, &|| Rid::new(0).to_variant()),
            VariantType::Object => Self::init_data_property_raw_variant(data, value, property_data),
            VariantType::Callable => Self::init_data_property_raw::<Callable>(data, value, property_data, &|| Callable::from_fn("NullFn", |_|{Ok(Variant::nil())}).to_variant()),
            VariantType::Signal => Self::init_data_property_raw::<Signal>(data, value, property_data, &|| Signal::invalid().to_variant()),
            VariantType::Dictionary => Self::init_data_property_raw_variant(data, value, property_data),
            VariantType::Array => Self::init_data_property_raw_variant(data, value, property_data),
            VariantType::PackedByteArray => Self::init_data_property_raw::<PackedByteArray>(data, value, property_data, &|| PackedByteArray::default().to_variant()),
            VariantType::PackedInt32Array => Self::init_data_property_raw::<PackedInt32Array>(data, value, property_data, &|| PackedInt32Array::default().to_variant()),
            VariantType::PackedInt64Array => Self::init_data_property_raw::<PackedInt64Array>(data, value, property_data, &|| PackedInt64Array::default().to_variant()),
            VariantType::PackedFloat32Array => Self::init_data_property_raw::<PackedFloat32Array>(data, value, property_data, &|| PackedFloat32Array::default().to_variant()),
            VariantType::PackedFloat64Array => Self::init_data_property_raw::<PackedFloat64Array>(data, value, property_data, &|| PackedFloat64Array::default().to_variant()),
            VariantType::PackedStringArray => Self::init_data_property_raw::<PackedStringArray>(data, value, property_data, &|| PackedStringArray::default().to_variant()),
            VariantType::PackedVector2Array => Self::init_data_property_raw::<PackedVector2Array>(data, value, property_data, &|| PackedVector2Array::default().to_variant()),
            VariantType::PackedVector3Array => Self::init_data_property_raw::<PackedVector3Array>(data, value, property_data, &|| PackedVector3Array::default().to_variant()),
            VariantType::PackedColorArray => Self::init_data_property_raw::<PackedColorArray>(data, value, property_data, &|| PackedColorArray::default().to_variant()),
        }
    }

    fn init_data_property_raw<T: FromGodot + ToGodot + Debug + Clone>(
        data: &mut [u8],
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
            let param_ptr:*mut u8 = &mut (*data)[property_data.offset];
            let param_slice = std::slice::from_raw_parts_mut(param_ptr, size_of::<T>());
            let value_ptr:*const T = &default_value.to::<T>();
            let value_slice = std::slice::from_raw_parts(value_ptr as *const u8, size_of::<T>());
            param_slice.copy_from_slice(value_slice);
        }
    }

    fn init_data_property_raw_variant(
        data: &mut [u8],
        value: Variant,
        property_data: &ComponetProperty,
    ) {
        let default_value = if value != Variant::nil() {
            value
        } else {
            Variant::default()
        };
        unsafe {
            let param_ptr:*mut u8 = &mut (*data)[property_data.offset];
            let param_slice = std::slice::from_raw_parts_mut(param_ptr, size_of::<Variant>());
            let value_ptr:*const Variant = &default_value;
            let value_slice = std::slice::from_raw_parts(value_ptr as *const u8, size_of::<Variant>());
            param_slice.copy_from_slice(value_slice);
        }
    }
    
    pub(crate) fn set_data_property_raw<T: FromGodot + ToGodot + Debug + Clone>(
        data: &mut [u8],
        value: Variant,
        offset: usize,
    ) {
        let param_ref = unsafe {
            NonNull::new_unchecked(&mut (*data)[offset])
                .cast::<T>()
                .as_mut()
        };
        *param_ref = value.to::<T>();
    }
    
    // Sets the property of the given data to it's type from a variant
    pub(crate) fn set_data_property(
        data: &mut [u8],
        value: Variant,
        property_data: &ComponetProperty,
    ) {
        match property_data.gd_type_id {
            VariantType::Nil => panic!("Can't set \"Nil\" type in component"),
            VariantType::Bool => Self::set_data_property_raw::<bool>(data, value, property_data.offset),
            VariantType::Int => Self::set_data_property_raw::<i32>(data, value, property_data.offset),
            VariantType::Float => Self::set_data_property_raw::<f32>(data, value, property_data.offset),
            VariantType::String => Self::set_data_property_raw::<GString>(data, value, property_data.offset),
            VariantType::Vector2 => Self::set_data_property_raw::<Vector2>(data, value, property_data.offset),
            VariantType::Vector2i => Self::set_data_property_raw::<Vector2i>(data, value, property_data.offset),
            VariantType::Rect2 => Self::set_data_property_raw::<Rect2>(data, value, property_data.offset),
            VariantType::Rect2i => Self::set_data_property_raw::<Rect2i>(data, value, property_data.offset),
            VariantType::Vector3 => Self::set_data_property_raw::<Vector3>(data, value, property_data.offset),
            VariantType::Vector3i => Self::set_data_property_raw::<Vector3i>(data, value, property_data.offset),
            VariantType::Transform2D => Self::set_data_property_raw::<Transform2D>(data, value, property_data.offset),
            VariantType::Vector4 => Self::set_data_property_raw::<Vector4>(data, value, property_data.offset),
            VariantType::Vector4i => Self::set_data_property_raw::<Vector4i>(data, value, property_data.offset),
            VariantType::Plane => Self::set_data_property_raw::<Plane>(data, value, property_data.offset),
            VariantType::Quaternion => Self::set_data_property_raw::<Quaternion>(data, value, property_data.offset),
            VariantType::Aabb => Self::set_data_property_raw::<Aabb>(data, value, property_data.offset),
            VariantType::Basis => Self::set_data_property_raw::<Basis>(data, value, property_data.offset),
            VariantType::Transform3D => Self::set_data_property_raw::<Transform3D>(data, value, property_data.offset),
            VariantType::Projection => Self::set_data_property_raw::<Projection>(data, value, property_data.offset),
            VariantType::Color => Self::set_data_property_raw::<Color>(data, value, property_data.offset),
            VariantType::StringName => Self::set_data_property_raw::<StringName>(data, value, property_data.offset),
            VariantType::NodePath => Self::set_data_property_raw::<NodePath>(data, value, property_data.offset),
            VariantType::Rid => Self::set_data_property_raw::<Rid>(data, value, property_data.offset),
            VariantType::Object => Self::set_data_property_raw_variant(data, value, property_data.offset),
            VariantType::Callable => Self::set_data_property_raw::<Callable>(data, value, property_data.offset),
            VariantType::Signal => Self::set_data_property_raw::<Signal>(data, value, property_data.offset),
            VariantType::Dictionary => Self::set_data_property_raw_variant(data, value, property_data.offset),
            VariantType::Array => Self::set_data_property_raw_variant(data, value, property_data.offset),
            VariantType::PackedByteArray => Self::set_data_property_raw::<PackedByteArray>(data, value, property_data.offset),
            VariantType::PackedInt32Array => Self::set_data_property_raw::<PackedInt32Array>(data, value, property_data.offset),
            VariantType::PackedInt64Array => Self::set_data_property_raw::<PackedInt64Array>(data, value, property_data.offset),
            VariantType::PackedFloat32Array => Self::set_data_property_raw::<PackedFloat32Array>(data, value, property_data.offset),
            VariantType::PackedFloat64Array => Self::set_data_property_raw::<PackedFloat64Array>(data, value, property_data.offset),
            VariantType::PackedStringArray => Self::set_data_property_raw::<PackedStringArray>(data, value, property_data.offset),
            VariantType::PackedVector2Array => Self::set_data_property_raw::<PackedVector2Array>(data, value, property_data.offset),
            VariantType::PackedVector3Array => Self::set_data_property_raw::<PackedVector3Array>(data, value, property_data.offset),
            VariantType::PackedColorArray => Self::set_data_property_raw::<PackedColorArray>(data, value, property_data.offset),
        }
    }
    
    /// Sets the property of the given data to variant
    fn set_data_property_raw_variant(
        data: &mut [u8],
        value: Variant,
        offset: usize,
    ) {
        let param_ref = unsafe {
            NonNull::new_unchecked(&mut (*data)[offset])
                .cast::<Variant>()
                .as_mut()
        };
        *param_ref = value;
    }

    pub(crate) fn new_default_data_getter(entity:EntityId) -> Box<dyn Fn(&Self) -> NonNull<[u8]>> {
        Box::new(move |this| {
            let value = unsafe { flecs::ecs_get_mut_id(
                this.world.bind().world.raw(),
                entity,
                this.get_flecs_id(),
            ) };
            unsafe { NonNull::new_unchecked(
                std::slice::from_raw_parts_mut(
                    value as *mut u8,
                    this.component_definition.layout.size(),
                ) 
            ) }
        })
    }

    pub(crate) fn new_empty_data_getter() -> Box<dyn Fn(&Self) -> NonNull<[u8]>> {
        Box::new(|_this| { unsafe { 
            NonNull::new_unchecked(&mut [])}
        })
    }

    fn get_data(&self) -> NonNull<[u8]> {
        ( &(*self.get_data_fn_ptr) )(self)
    }

    /// Returns the Flecs ID of this component's type.
    pub(crate) fn get_flecs_id(&self) -> EntityId {
        self.component_definition.flecs_id
    }

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
        
        let data = unsafe { self.get_data().as_mut() };
        let value = Self::get_data_property(data, &property_meta);

        if
            value == Variant::nil()
            && property_meta.gd_type_id != VariantType::Object
        {
            show_error!(
                "Failed to get property",
                "No property named \"{}\" in component of type \"{}\"",
                property,
                self.get_component_type_name(),
            );
        }

        value
    }

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
            if property_type == VariantType::Nil {
                break 'cancel_type_check
            }
            if value_type != property_type {
                if
                    property_type == VariantType::Object
                        && value_type == VariantType::Nil
                { break 'cancel_type_check }

                show_error!(
                    "Failed to set property",
                    "Expected type {:?}, but got type {:?}.",
                    property_type,
                    value_type,
                );
                return true;
            }
        }
        
        let data = unsafe { self.get_data().as_mut() };
        Self::set_data_property(data, value, &property_meta);

        return true;
    }

    // Similar to [_set_property], except it does not call the destructor.
    pub(crate) fn _initialize_property(
        data:&mut [u8],
        description:&ComponetDefinition,
        property:StringName,
        value:Variant,
    ) -> bool {
        let Some(property_data) = description
            .get_property(&property) else {
                show_error!(
                    "Property initialization failed",
                    "Can't write to {} in {{component}}. Component has no property with that name",
                    property,
                );
                return false;
            };

        let value_type = value.get_type();
        let property_type = property_data.gd_type_id;
        if property_type != VariantType::Nil {
            if value_type != property_type && value_type != VariantType::Nil {
                show_error!(
                    "Failed to set property",
                    "Expected type {:?}, but got type {:?}.",
                    property_type,
                    value_type,
                );
                return true;
            }
        }

        Self::init_data_property(data, value, property_data);

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
    
    fn get_property(&self, property: StringName) -> Option<Variant> {
        Some(self._get_property(property))
    }

    fn set_property(&mut self, property: StringName, v:Variant) -> bool{
        self._set_property(property, v)
    }
}