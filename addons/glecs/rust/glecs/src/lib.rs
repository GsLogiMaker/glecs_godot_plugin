
use std::alloc::Layout;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::default;
use std::ffi::c_void;
use std::fmt::Debug;
use std::hash::Hash;
use std::mem::ManuallyDrop;
use std::mem::size_of;
use std::pin::Pin;
use std::rc::Rc;
use std::slice::from_raw_parts;
use std::slice::from_raw_parts_mut;
use std::sync::Mutex;

use flecs::ecs_get_mut_id;
use flecs::EntityId;
use flecs::Iter;
use flecs::ecs_set_threads;
use flecs::world::World as FlWorld;
use flecs::Entity as FlEntity;
use flecs::TermBuilder;

use godot::engine;
use godot::engine::utilities::push_error;
use godot::engine::GdScript;
use godot::engine::Script;
use godot::obj::EngineClass;
use godot::obj::EngineEnum;
use godot::obj::WithBaseField;
use godot::prelude::*;
use godot::engine::Node;
use godot::engine::INode;
use godot::engine::Object;
use godot::engine::Engine;
use godot::engine::global::MethodFlags;
use godot::engine::global::Error as GdError;

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


#[derive(GodotClass)]
#[class(base=Object)]
struct _BaseGEEntity {
    #[base] base: Base<Object>,
    /// The world this entity is from.
    world: Gd<_BaseGEWorld>,
    /// The ID of this entity.
    id: EntityId,
}
#[godot_api]
impl _BaseGEEntity {
    #[func]
    fn get_component(&mut self, component:Gd<Script>) -> Option<Gd<_BaseGEComponent>> {
        let world = self.world.bind();

        // Get component description
        let Some(component_definition) = world
            .get_component_description(&component)
            else {
                godot_error!(
                    "Failed to get component from entity. Component {} has not been added to entity {}.",
                    component,
                    self.to_gd(),
                );
                return None;
            };

        // Get flecs entity
        let component_symbol = component_definition.name.to_string();
        let Some(mut entt) = world.world.find_entity(self.id)
            else { 
                godot_error!(
                    "Failed to get component from entity. Entity {} was freed.",
                    self.to_gd(),
                );
                return None;
            };
        
        // Get component data
        if !entt.has_id(component_definition.flecs_id) {
            godot_error!(
                "Failed to get component from entity. Component {} has not been added to entity {}.",
                    component,
                    self.to_gd(),
            );
            return None;
        }
        let component_data = entt.get_mut_dynamic(&component_symbol);

        
        let mut comp = Gd::from_init_fn(|base| {
            let base_comp = _BaseGEComponent {
                base,
                flecs_id: component_definition.flecs_id,
                data: component_data,
                component_definition,
            };
            base_comp
        });
        comp.bind_mut().base_mut().set_script(component.to_variant());

        Some(comp)
    }

    fn add_component(&mut self, component:Gd<Script>) -> Option<Gd<_BaseGEComponent>> {
        let component_definition = self.world
            .bind_mut()
            .get_or_add_component(&component);

        let world = self.world.bind();

        unsafe {
            flecs::ecs_add_id(
                world.world.raw(),
                self.id,
                component_definition.flecs_id,
            )
        };

        // Get component data
        let Some(mut entt) = world.world.find_entity(self.id)
            else { 
                godot_error!(
                    "Failed to get component from entity. Entity {} was freed.",
                    self.to_gd(),
                );
                return None;
            };
        if !entt.has_id(component_definition.flecs_id) {
            godot_error!(
                "Failed to get component from entity. Component {} has not been added to entity {}.",
                    component,
                    self.to_gd(),
            );
            return None;
        }
        let component_data = entt.get_mut_dynamic(
            &component_definition.name.to_string()
        );

        // Initialize component properties
        // TODO: Initialize properties in deterministic order
        for property_name in component_definition.parameters.keys() {
            // TODO: Get default values of properties
            let default_value = Variant::nil();
            _BaseGEComponent::_initialize_property(
                component_data,
                component_definition.as_ref(),
                property_name.clone(),
                default_value,
            );
        }

        let mut comp = Gd::from_init_fn(|base| {
            let base_comp = _BaseGEComponent {
                base,
                flecs_id: component_definition.flecs_id,
                data: component_data,
                component_definition,
            };
            base_comp
        });
        comp.bind_mut().base_mut().set_script(component.to_variant());

        Some(comp)
    }

    fn flecs_free(&self) {
        let entt = self.world.bind().world.find_entity(self.id).unwrap();
        entt.destruct()
    }
}
#[godot_api]
impl IObject for _BaseGEEntity {
    fn on_notification(&mut self, what: engine::notify::ObjectNotification) {
        match what {
            engine::notify::ObjectNotification::Predelete => {
                self.flecs_free()
            },
            _ => {},
        }
    }
}

/// An ECS component.
#[derive(GodotClass)]
#[class(base=RefCounted)]
struct _BaseGEComponent {
    #[base] base: Base<RefCounted>,
    /// The Flecs component ID for the type of this component.
    flecs_id: EntityId,
    data: *mut [u8],
    component_definition: Rc<ScriptComponetDefinition>,
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

    fn _get_property(&self, property:StringName) -> Variant {
        let Some(property_data) = self
            .component_definition
            .parameters
            .get(&property)
            else {
                godot_error!(
                    "Failed to get property. No property named \"{}\" in component of type \"{}\"",
                    property,
                    self.get_component_type_name()
                );
                return Variant::nil();
            };
        
        fn get_param<T: ToGodot + Clone + Debug>(
            data:*mut [u8],
            property_data: &ScriptComponetProperty
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

    fn _set_property(&mut self, property:StringName, value:Variant) -> bool {
        let Some(property_data) = self
            .component_definition
            .parameters
            .get(&property) else {
                godot_error!(
                    "Failed to set property. No property named \"{}\" in component of type \"{}\"",
                    property,
                    self.get_component_type_name(),
                );
                return false;
            };

        if value.get_type() != property_data.gd_type_id {
            godot_error!(
                "Failed to set property. Expected type {:?}, but got type {:?}",
                property_data.gd_type_id,
                value.get_type(),
            );
            return true;
        }
        
        fn set_param<T: FromGodot + ToGodot + Debug + Clone>(
            data:*mut [u8],
            value: Variant,
            property_data: &ScriptComponetProperty,
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
            VariantType::Object => todo!("Object doesn't support conversion to variant or copying"), /* get_param::<Object>(self.data, property_data), */
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
    fn _initialize_property(
        data:&mut [u8],
        description:&ScriptComponetDefinition,
        property:StringName,
        value:Variant, // TODO: Utilize the initialization value
    ) -> bool {
        let Some(property_data) = description
            .parameters
            .get(&property) else {
                godot_error!(
                    "Can't write to {} in {{component}}. Component has to property with that name",
                    property,
                );
                return false;
            };
        
        fn init_param<T: FromGodot + ToGodot + Debug + Default + Clone>(
            data:*mut [u8],
            value: Variant,
            property_data: &ScriptComponetProperty,
        ) {
             let default_value = if value != Variant::nil() {
                value
            } else {
                Variant::from(T::default())
            };
            unsafe {
                let param_ptr:*mut u8 = &mut (*data)[property_data.offset];
                let param_slice = from_raw_parts_mut(param_ptr, size_of::<T>());
                let value_ptr:*const T = &default_value.to::<T>().clone();
                let value_slice = from_raw_parts(value_ptr as *const u8, size_of::<T>());
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
}
#[godot_api]
impl IRefCounted for _BaseGEComponent {
    // fn get_property(&self, property: StringName) -> Option<Variant> {
    //     Some(self._get_property(property))
    // }

    // fn set_property(&mut self, property: StringName, v:Variant) -> bool{
    //     self._set_property(property, v)
    // }
}

/// The metadata regarding a component's structure.
#[derive(Debug, Clone)]
struct ScriptComponetDefinition {
    name: StringName,
    parameters: HashMap<StringName, ScriptComponetProperty>,
    flecs_id: EntityId,
    script_id: InstanceId,
    layout: Layout,
} impl ScriptComponetDefinition {
    fn new(
        mut component: Gd<Script>,
        mut world: &mut _BaseGEWorld,
    ) -> Self {
        let script_properties = component
            .get_script_property_list();

        let mut component_properties = HashMap::default();
        let mut offset = 0;
        let mut i = 0;
        while i != script_properties.len() {
            let property = script_properties.get(i);
            let property_type = property
                .get(StringName::from("type"))
                .unwrap()
                .to::<VariantType>();
            if property_type == VariantType::Nil {
                i += 1;
                continue;
            }
            let property_name:StringName = property
                .get(StringName::from("name"))
                .unwrap()
                .to::<String>()
                .into();

            component_properties.insert(
                property_name.clone(),
                ScriptComponetProperty {
                    name: property_name,
                    gd_type_id: property_type,
                    offset,
                },
            );

            offset += TYPE_SIZES[property_type as usize];
            i += 1;
        }

        let name = component.to_string();
        let layout = _BaseGEWorld::layout_from_properties(&component_properties);
        let mut script_component = Self {
            name: name.clone().into(),
            parameters: component_properties,
            flecs_id: 0,
            script_id: component.instance_id(),
            layout,
        };
        script_component.flecs_id = world.world
            .component_dynamic(name, layout);
        
        script_component
    }
}

/// The definition for one property in a component's definition.
#[derive(Debug, Clone)]
struct ScriptComponetProperty {
    name: StringName,
    gd_type_id: VariantType,
    offset: usize,
} impl Default for ScriptComponetProperty {
    fn default() -> Self {
        Self { 
            name: Default::default(),
            gd_type_id: VariantType::Nil,
            offset: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct ScriptSystemContext {
    callable: Callable,
    terms: Array<Gd<Script>>,
    /// The arguments passed to the system.
    system_args: Array<Variant>,
    /// Holds the accesses stored in `sysatem_args` for quicker access.
    term_accesses: Box<[Gd<_BaseGEComponent>]>,
    world: Gd<_BaseGEWorld>,
}

#[derive(GodotClass)]
#[class(base=Node)]
struct _BaseGEWorld {
    #[base] node: Base<Node>,
    world: FlWorld,
    component_definitions: ComponentDefinitions,
    system_contexts: LinkedList<Pin<Box<ScriptSystemContext>>>,
    gd_entity_map: HashMap<EntityId, Gd<_BaseGEEntity>>,
}
#[godot_api]
impl _BaseGEWorld {
    #[func]
    fn _world_process(&mut self, delta:f32) {
        self.world.progress(delta);
    }

    /// Returns the name of the Script that was registered with the world.
    #[func]
    fn get_script_component_name(
        &self,
        script: Gd<Script>,
    ) -> StringName {
        self.component_definitions.get(&script.instance_id())
            .ok_or_else(|| { format!(
                "Can't find component '{}' in entity. That component hasn't been registered with the world.",
                script,
            )})
            .unwrap()
            .name
            .clone()
    }

    /// Creates a new entity in the world.
    #[func]
    fn _new_entity(
        &mut self,
        with_components:Array<Gd<Script>>,
    ) -> Gd<_BaseGEEntity> {
        let mut entity = self.world.entity();
        let mut i = 0;
        while i != with_components.len() {
            let mut script = with_components.get(i);

            let comp_def = self
                .get_or_add_component(&script);
            entity = entity.add_id(comp_def.flecs_id);

            let data = entity.get_mut_dynamic(&comp_def.name.to_string());

            i += 1;

            // Initialize component properties
            // TODO: Initialize properties in deterministic order
            for property_name in comp_def.parameters.keys() {
                // TODO: Get default values of properties
                let default_value = script
                    .get_property_default_value(property_name.clone());
                _BaseGEComponent::_initialize_property(
                    data,
                    comp_def.as_ref(),
                    property_name.clone(),
                    default_value,
                );
            }
        }

        let gd_entity = Gd::from_init_fn(|base| {
            _BaseGEEntity {
                base,
                world: self.to_gd(),
                id: entity.id(),
            }
        });
        self.gd_entity_map.insert(entity.id(), gd_entity);
        gd_entity
    }

    

    // Defines a new system to be run in the world.
    #[func]
    fn _add_system(&mut self, callable: Callable, terms: Array<Gd<Script>>) {
        // Create term list
        let mut term_ids = vec![];
        for i in 0..terms.len() {
            let script = terms.get(i);
            let comp_def = self
                .component_definitions
                .get(script.instance_id()).unwrap();
            term_ids.push(comp_def.flecs_id);
        }

        // Create component accesses
        let mut system_args = array![];
        let mut tarm_accesses: Vec<Gd<_BaseGEComponent>> = vec![];
        for term_i in 0..terms.len() as usize {
            let term_script = terms.get(term_i).clone();
            let mut compopnent_access = Gd
                ::<_BaseGEComponent>
                ::from_init_fn(|base| {
                    _BaseGEComponent {
                        base,
                        flecs_id: term_ids[term_i],
                        data: &mut [],
                        component_definition: self
                            .get_or_add_component(&term_script),
                    }
                });
            compopnent_access.set_script(term_script.to_variant());
            system_args.push(compopnent_access.to_variant());
            tarm_accesses.push(compopnent_access);
        }
        let term_args_fast = tarm_accesses
            .into_boxed_slice();

        // Create contex
        self.system_contexts.push_back(Pin::new(Box::new(
            ScriptSystemContext {
                system_args: system_args,
                term_accesses: term_args_fast,
                callable: callable.clone(),
                terms: terms,
                world: self.to_gd(),
            }
        )));
        let context_ptr:*mut Pin<Box<ScriptSystemContext>> = self
            .system_contexts
            .back_mut()
            .unwrap();

        // Create system
        let mut sys = self.world
            .system()
            .context_ptr(context_ptr.cast::<c_void>());
        for id in term_ids.iter() {
            sys = sys.term_dynamic(*id);
        }

        // System body
        sys.iter(Self::system_iteration);
    }

    fn get_component_description(
        &self,
        key:impl Into<ComponentDefinitionsMapKey>,
    ) -> Option<Rc<ScriptComponetDefinition>> {
        self.component_definitions.get(key)
    }

    fn get_or_add_component(
        &mut self,
        key: &Gd<Script>,
    ) -> Rc<ScriptComponetDefinition> {
        let value = ComponentDefinitionsMapKey
            ::from(key)
            .get_value(&self.component_definitions);
        match value {
            Some(value) => value,
            None => {
                let def = ScriptComponetDefinition::new(
                    key.clone(),
                    self,
                );
                self.component_definitions.insert(def)
            }
        }
    }

    fn layout_from_properties(
        parameters: &HashMap<StringName, ScriptComponetProperty>,
    ) -> Layout {
        let mut size = 0;
        for (_name, property) in parameters {
            size += TYPE_SIZES[property.gd_type_id as usize];
        }
        Layout::from_size_align(size, 8).unwrap()
    }

    fn system_iteration(iter:&Iter) {
        // Get context
        let context = unsafe {
            (iter as *const Iter)
                .cast_mut()
                .as_mut()
                .unwrap()
                .get_context_mut::<Pin<Box<ScriptSystemContext>>>()
        };

        for entity_i in 0..(iter.count() as usize) {
            // Create components arguments
            for field_i in 0i32..iter.field_count() {
                let mut column = iter
                    .field_dynamic(field_i+1);
                let data:*mut [u8] = column.get_mut(entity_i);

                context.term_accesses[field_i as usize]
                    .bind_mut()
                    .data = data;
            }
            
            let _result = context.callable.callv(
                context.system_args.clone()
            );
        }
    }
}

// 2_205_783
//   660_882

#[godot_api]
impl INode for _BaseGEWorld {
    fn init(node: Base<Node>) -> Self {
        let world = FlWorld::new();
        unsafe {ecs_set_threads(world.raw(), 1)};
        Self {
            node,
            world: world,
            component_definitions: Default::default(),
            system_contexts: Default::default(),
        }
    }

    // fn physics_process(&mut self, delta:f64) {
    //     self.world.progress(delta as f32);
    // }
}

#[derive(Eq, PartialEq, Hash)]
enum ComponentDefinitionsMapKey {
    Name(StringName),
    FlecsId(EntityId),
    ScriptId(InstanceId),
} impl ComponentDefinitionsMapKey {
    fn get_index(&self, d:&ComponentDefinitions) -> usize {
        use ComponentDefinitionsMapKey::Name;
        use ComponentDefinitionsMapKey::FlecsId;
        use ComponentDefinitionsMapKey::ScriptId;
        self.get_index_maybe(d).unwrap_or_else(|| {
            match self {
                Name(k) => !unimplemented!(),
                FlecsId(k) => !unimplemented!(),
                ScriptId(k) => {
                    let script:Gd<Script> = Gd::from_instance_id(*k);
                    let msg = format!(
                        "No component has been registered with script \"{}\"",
                        script,
                    );
                    godot_error!("{msg}");
                    panic!("{msg}");
                },
            }
        })
    }

    fn get_index_maybe(&self, d:&ComponentDefinitions) -> Option<usize> {
        use ComponentDefinitionsMapKey::Name;
        use ComponentDefinitionsMapKey::FlecsId;
        use ComponentDefinitionsMapKey::ScriptId;
        match self {
            Name(k) => d.name_map.get(k).map(|x| *x),
            FlecsId(k) => d.flecs_id_map.get(k).map(|x| *x),
            ScriptId(k) => d.script_id_map.get(k).map(|x| *x),
        }
    }

    fn get_value(&self, d:&ComponentDefinitions) -> Option<Rc<ScriptComponetDefinition>> {
        d.data.get(self.get_index_maybe(d)?).map(|x| {
            x.clone()
        })
    }
} impl From<StringName> for ComponentDefinitionsMapKey {
    fn from(value: StringName) -> Self {
        ComponentDefinitionsMapKey::Name(value)
    }
} impl From<EntityId> for ComponentDefinitionsMapKey {
    fn from(value: EntityId) -> Self {
        ComponentDefinitionsMapKey::FlecsId(value)
    }
} impl From<Gd<Script>> for ComponentDefinitionsMapKey {
    fn from(value: Gd<Script>) -> Self {
        ComponentDefinitionsMapKey::ScriptId(value.instance_id())
    }
} impl From<&Gd<Script>> for ComponentDefinitionsMapKey {
    fn from(value: &Gd<Script>) -> Self {
        ComponentDefinitionsMapKey::ScriptId(value.instance_id())
    }
} impl From<InstanceId> for ComponentDefinitionsMapKey {
    fn from(value: InstanceId) -> Self {
        ComponentDefinitionsMapKey::ScriptId(value)
    }
} impl From<&StringName> for ComponentDefinitionsMapKey {
    fn from(value: &StringName) -> Self {
        ComponentDefinitionsMapKey::Name(value.clone())
    }
} impl From<&EntityId> for ComponentDefinitionsMapKey {
    fn from(value: &EntityId) -> Self {
        ComponentDefinitionsMapKey::FlecsId(value.clone())
    }
} impl From<&InstanceId> for ComponentDefinitionsMapKey {
    fn from(value: &InstanceId) -> Self {
        ComponentDefinitionsMapKey::ScriptId(value.clone())
    }
}

#[derive(Debug, Default)]
struct ComponentDefinitions {
    data: Vec<Rc<ScriptComponetDefinition>>,
    name_map:HashMap<StringName, usize>,
    flecs_id_map:HashMap<EntityId, usize>,
    script_id_map:HashMap<InstanceId, usize>,
} impl ComponentDefinitions {
    fn add_mapping(
        &mut self,
        index: usize,
        name_map: StringName,
        flecs_id_map: EntityId,
        script_id_map: InstanceId,
    ) {
        self.name_map.insert(name_map, index);
        self.flecs_id_map.insert(flecs_id_map, index);
        self.script_id_map.insert(script_id_map, index);
        for key in &self.script_id_map {
            godot_print!("ADD NEW COMP {}", key.0);
        }
    }

    fn insert(&mut self, element:ScriptComponetDefinition) -> Rc<ScriptComponetDefinition> {
        let len: usize = self.data.len();
        self.add_mapping(
            len,
            element.name.clone(),
            element.flecs_id,
            element.script_id.clone(),
        );
        let rc = Rc::new(element);
        self.data.push(rc.clone());
        rc
    }

    fn get(
        &self,
        key:impl Into<ComponentDefinitionsMapKey>,
    ) -> Option<Rc<ScriptComponetDefinition>> {
        let x = key.into();
        x.get_value(self)
    }

    fn has(&self, map:impl Into<ComponentDefinitionsMapKey>) -> bool{
        map.into().get_index_maybe(self).is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::{alloc::Layout, mem::size_of_val};

    use flecs::{TermBuilder, Entity};

    use super::*;

    #[test]
    fn sizes() {
        let value = vec![1, 2, 3];
        let value2:HashMap<String, ()> = HashMap::default();
        let size = size_of_val(&|| {5});
        let size2 = size_of_val(&|| {
            let a = &value;
            let b = &value2;
        });
        println!("&||{{5}} {size}");
        println!("&|| {{let a = &value;}} {size2}");
    }

    #[test]
    fn it_works() {
        let mut world = FlWorld::new();
        let run = world.component_dynamic(
            "Run",
            Layout::for_value(&0i64)
        );
        world.entity().set_dynamic("Run", &30i64.to_le_bytes());

        world.system().term_dynamic(run).iter(|iter|{
            let run_column = iter.field_dynamic(1);
            for i in 0..(iter.count() as usize) {
                let run:*const [u8] = run_column.get(i);
                let run = unsafe {run.cast::<i64>().as_ref().unwrap()};
                println!("run: {run:?}");
            }
        });

        world.progress(0.1);

        // let entity = world
        //     .entity()
        //     .add_dynamic("Run")
        //     .set_dynamic("Run", &1i64.to_le_bytes());
        // let entity2 = world
        //     .entity()
        //     .add_dynamic("Run")
        //     .set_dynamic("Run", &2i64.to_le_bytes());
        
        // let query = world.query().term_dynamic(run).build();

        // query.iter(|comp| {
        //     let column = comp.field_dynamic(1);
        //     let a = column.get_count();
        //     dbg!(&column.get(2));
        // });
    }
}
