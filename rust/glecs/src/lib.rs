
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
use std::sync::Mutex;

use flecs::ecs_get_mut_id;
use flecs::EntityId;
use flecs::Iter;
use flecs::ecs_set_threads;
use flecs::world::World as FlWorld;
use flecs::Entity as FlEntity;
use flecs::TermBuilder;

use godot::engine;
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

const TYPE_NIL:i32 = 0;
const TYPE_BOOL:i32 = 1;
const TYPE_INT:i32 = 2;
const TYPE_FLOAT:i32 = 3;
const TYPE_STRING:i32 = 4;
const TYPE_VECTOR2:i32 = 5;
const TYPE_VECTOR2I:i32 = 6;
const TYPE_RECT2:i32 = 7;
const TYPE_RECT2I:i32 = 8;
const TYPE_VECTOR3:i32 = 9;
const TYPE_VECTOR3I:i32 = 10;
const TYPE_TRANSFORM2D:i32 = 11;
const TYPE_VECTOR4:i32 = 12;
const TYPE_VECTOR4I:i32 = 13;
const TYPE_PLANE:i32 = 14;
const TYPE_QUATERNION:i32 = 15;
const TYPE_AABB:i32 = 16;
const TYPE_BASIS:i32 = 17;
const TYPE_TRANSFORM3D:i32 = 18;
const TYPE_PROJECTION:i32 = 19;
const TYPE_COLOR:i32 = 20;
const TYPE_STRING_NAME:i32 = 21;
const TYPE_NODE_PATH:i32 = 22;
const TYPE_RID:i32 = 23;
const TYPE_OBJECT:i32 = 24;
const TYPE_CALLABLE:i32 = 25;
const TYPE_SIGNAL:i32 = 26;
const TYPE_DICTIONARY:i32 = 27;
const TYPE_ARRAY:i32 = 28;
const TYPE_PACKED_BYTE_ARRAY:i32 = 29;
const TYPE_PACKED_INT32_ARRAY:i32 = 20;
const TYPE_PACKED_INT64_ARRAY:i32 = 31;
const TYPE_PACKED_FLOAT32_ARRAY:i32 = 32;
const TYPE_PACKED_FLOAT64_ARRAY:i32 = 33;
const TYPE_PACKED_STRING_ARRAY:i32 = 34;
const TYPE_PACKED_VECTOR2_ARRAY:i32 = 35;
const TYPE_PACKED_VECTOR3_ARRAY:i32 = 36;
const TYPE_PACKED_COLOR_ARRAY:i32 = 37;
const TYPE_MAX:i32 = 38;

const TYPE_SIZES:&'static [usize] = &[
    /* NIL */ 0,
    /* BOOL */ size_of::<bool>(),
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
#[class(base=RefCounted)]
struct _BaseGEEntity {
    #[base] base: Base<RefCounted>,
    /// The world this entity is from.
    world: Gd<_BaseGEWorld>,
    /// The ID of this entity.
    id: EntityId,
}
#[godot_api]
impl _BaseGEEntity {
    #[func]
    fn get_component(&mut self, component:Gd<Script>) -> Gd<_BaseGEComponent> {
        let world_gd = self.world.clone();
        let world = self.world.bind();

        let component_definition = world
            .get_component_description(component.instance_id())
            .unwrap();
        let component_symbol = component_definition.name.to_string();
        let mut entt = world.world.find_entity(self.id)
            .expect("TODO: Add err msg");
        let component_data = entt.get_mut_dynamic(&component_symbol);

        
        let mut comp = Gd::from_init_fn(|base| {
            let base_comp = _BaseGEComponent {
                base,
                flecs_id: component_definition.flecs_id,
                data: component_data,
                world: world_gd,
            };
            base_comp
        });
        comp.bind_mut().base_mut().set_script(component.to_variant());
        comp
    }

    #[func]
    fn set_component(&mut self, component:Gd<Script>, value:Variant) {
        let world = self.world.bind_mut();
        let component_name = world
            .get_script_component_name(component);

        let entt = world.world.find_entity(self.id)
            .expect("TODO: Add err msg");


		// let component_id = unsafe { ecs_get_mut_id(world.world, self.entity, comp_id) };
        // component.
        // flecs::bindings::ecs_lookup(world.world.raw(), )
        // world.world.find_entity(self.id)
        //     .unwrap()
        //     .comp
        // world.world.set_component(self.id, comp, data)
    }
}

#[derive(GodotClass)]
#[class(base=RefCounted)]
/// An ECS component.
struct _BaseGEComponent {
    #[base] base: Base<RefCounted>,
    /// The Flecs component ID for the type of this component.
    flecs_id: EntityId,
    data: *mut [u8],
    world: Gd<_BaseGEWorld>,
}
#[godot_api]
impl _BaseGEComponent {
    #[func]
    fn get(&self, property: StringName) -> Variant {
        self._get_property(property)
        // Variant::from(5)
    }

    #[func]
    fn set(&mut self, property: StringName, value:Variant) {
        self._set_property(property, value);
    }

    fn _get_property(&self, property:StringName) -> Variant {
        let world = self.world.bind();

        let description = world
            .get_component_description(self.flecs_id)
            .unwrap();

        let Some(property_data) = description
            .parameters
            .get(&property) else {
                return Variant::nil();
            };
        
        fn get_param<T: ToGodot + Clone>(
            data:*mut [u8],
            property_data: &ScriptComponetProperty
        ) -> Variant {
            unsafe {
                let param:*mut u8 = &mut (*data)[property_data.offset];
                let value = Variant::from((*param.cast::<T>()).clone());
                return value;
                // return Variant::nil();
            }
        }
        
        let value =  match property_data.gd_type_id {
            TYPE_BOOL => get_param::<bool>(self.data, property_data),
            TYPE_INT => get_param::<i32>(self.data, property_data),
            TYPE_FLOAT => get_param::<f32>(self.data, property_data),
            TYPE_STRING => get_param::<GString>(self.data, property_data),
            TYPE_VECTOR2 => get_param::<Vector2>(self.data, property_data),
            TYPE_VECTOR2I => get_param::<Vector2i>(self.data, property_data),
            TYPE_RECT2 => get_param::<Rect2>(self.data, property_data),
            TYPE_RECT2I => get_param::<Rect2i>(self.data, property_data),
            TYPE_VECTOR3 => get_param::<Vector3>(self.data, property_data),
            TYPE_VECTOR3I => get_param::<Vector3i>(self.data, property_data),
            TYPE_TRANSFORM2D => get_param::<Transform2D>(self.data, property_data),
            TYPE_VECTOR4 => get_param::<Vector4>(self.data, property_data),
            TYPE_VECTOR4I => get_param::<Vector4i>(self.data, property_data),
            TYPE_PLANE => get_param::<Plane>(self.data, property_data),
            TYPE_QUATERNION => get_param::<Quaternion>(self.data, property_data),
            TYPE_AABB => get_param::<Aabb>(self.data, property_data),
            TYPE_BASIS => get_param::<Basis>(self.data, property_data),
            TYPE_TRANSFORM3D => get_param::<Transform3D>(self.data, property_data),
            TYPE_PROJECTION => get_param::<Projection>(self.data, property_data),
            TYPE_COLOR => get_param::<Color>(self.data, property_data),
            TYPE_STRING_NAME => get_param::<StringName>(self.data, property_data),
            TYPE_NODE_PATH => get_param::<NodePath>(self.data, property_data),
            TYPE_RID => get_param::<Rid>(self.data, property_data),
            TYPE_OBJECT => todo!("Object doesn't support conversion to variant or copying"), /* get_param::<Object>(self.data, property_data), */
            TYPE_CALLABLE => get_param::<Callable>(self.data, property_data),
            TYPE_SIGNAL => get_param::<Signal>(self.data, property_data),
            TYPE_DICTIONARY => get_param::<Dictionary>(self.data, property_data),
            TYPE_ARRAY => todo!("Causes crashes"), /* get_param::<Array<Variant>>(self.data, property_data), */
            TYPE_PACKED_BYTE_ARRAY => get_param::<PackedByteArray>(self.data, property_data),
            TYPE_PACKED_INT32_ARRAY => get_param::<PackedInt32Array>(self.data, property_data),
            TYPE_PACKED_INT64_ARRAY => get_param::<PackedInt64Array>(self.data, property_data),
            TYPE_PACKED_FLOAT32_ARRAY => get_param::<PackedFloat32Array>(self.data, property_data),
            TYPE_PACKED_FLOAT64_ARRAY => get_param::<PackedFloat64Array>(self.data, property_data),
            TYPE_PACKED_STRING_ARRAY => get_param::<PackedStringArray>(self.data, property_data),
            TYPE_PACKED_VECTOR2_ARRAY => get_param::<PackedVector2Array>(self.data, property_data),
            TYPE_PACKED_VECTOR3_ARRAY => get_param::<PackedVector3Array>(self.data, property_data),
            TYPE_PACKED_COLOR_ARRAY => get_param::<PackedColorArray>(self.data, property_data),

            _ => todo!(),
        };
        value
        // return Variant::from(5);
    }

    fn _set_property(&mut self, property:StringName, value:Variant) -> bool {
        let world = self.world.bind();
        
        let description = world
            .get_component_description(self.flecs_id)
            .unwrap();

        let Some(property_data) = description
            .parameters
            .get(&property) else {
                godot_error!(
                    "Can't write to {} in {{component}}. Component has to property with that name",
                    property,
                );
                return false;
            };
        
        fn set_param<T: FromGodot + ToGodot + Debug + Clone>(
            data:*mut [u8],
            value: Variant,
            property_data: &ScriptComponetProperty,
        ) {
            unsafe {
                let param_raw:*mut u8 = &mut (*data)[property_data.offset];
                let param = param_raw.cast::<T>();
                {
                    let data_length = (&*data).len();
                    godot_print!(
                        "{} <= {} - {}",
                        size_of::<T>(),
                        data_length,
                        property_data.offset
                    );
                    assert!(size_of::<T>() <= data_length-property_data.offset);
                }
                *param = value.to::<T>().clone();
            }
        }
        
        match property_data.gd_type_id {
            TYPE_BOOL => set_param::<bool>(self.data, value, property_data),
            TYPE_INT => set_param::<i32>(self.data, value, property_data),
            TYPE_FLOAT => set_param::<f32>(self.data, value, property_data),
            TYPE_STRING => set_param::<GString>(self.data, value, property_data),
            TYPE_VECTOR2 => set_param::<Vector2>(self.data, value, property_data),
            TYPE_VECTOR2I => set_param::<Vector2i>(self.data, value, property_data),
            TYPE_RECT2 => set_param::<Rect2>(self.data, value, property_data),
            TYPE_RECT2I => set_param::<Rect2i>(self.data, value, property_data),
            TYPE_VECTOR3 => set_param::<Vector3>(self.data, value, property_data),
            TYPE_VECTOR3I => set_param::<Vector3i>(self.data, value, property_data),
            TYPE_TRANSFORM2D => set_param::<Transform2D>(self.data, value, property_data),
            TYPE_VECTOR4 => set_param::<Vector4>(self.data, value, property_data),
            TYPE_VECTOR4I => set_param::<Vector4i>(self.data, value, property_data),
            TYPE_PLANE => set_param::<Plane>(self.data, value, property_data),
            TYPE_QUATERNION => set_param::<Quaternion>(self.data, value, property_data),
            TYPE_AABB => set_param::<Aabb>(self.data, value, property_data),
            TYPE_BASIS => set_param::<Basis>(self.data, value, property_data),
            TYPE_TRANSFORM3D => set_param::<Transform3D>(self.data, value, property_data),
            TYPE_PROJECTION => set_param::<Projection>(self.data, value, property_data),
            TYPE_COLOR => set_param::<Color>(self.data, value, property_data),
            TYPE_STRING_NAME => set_param::<StringName>(self.data, value, property_data),
            TYPE_NODE_PATH => set_param::<NodePath>(self.data, value, property_data),
            TYPE_RID => set_param::<Rid>(self.data, value, property_data),
            TYPE_OBJECT => todo!("Object doesn't support conversion to variant or copying"), /* get_param::<Object>(self.data, property_data), */
            TYPE_CALLABLE => set_param::<Callable>(self.data, value, property_data),
            TYPE_SIGNAL => set_param::<Signal>(self.data, value, property_data),
            TYPE_DICTIONARY => set_param::<Dictionary>(self.data, value, property_data),
            TYPE_ARRAY => todo!("Causes crashes"), /* get_param::<Array<Variant>>(self.data, property_data), */
            TYPE_PACKED_BYTE_ARRAY => set_param::<PackedByteArray>(self.data, value, property_data),
            TYPE_PACKED_INT32_ARRAY => set_param::<PackedInt32Array>(self.data, value, property_data),
            TYPE_PACKED_INT64_ARRAY => set_param::<PackedInt64Array>(self.data, value, property_data),
            TYPE_PACKED_FLOAT32_ARRAY => set_param::<PackedFloat32Array>(self.data, value, property_data),
            TYPE_PACKED_FLOAT64_ARRAY => set_param::<PackedFloat64Array>(self.data, value, property_data),
            TYPE_PACKED_STRING_ARRAY => set_param::<PackedStringArray>(self.data, value, property_data),
            TYPE_PACKED_VECTOR2_ARRAY => set_param::<PackedVector2Array>(self.data, value, property_data),
            TYPE_PACKED_VECTOR3_ARRAY => set_param::<PackedVector3Array>(self.data, value, property_data),
            TYPE_PACKED_COLOR_ARRAY => set_param::<PackedColorArray>(self.data, value, property_data),

            _ => todo!(),
        }

        return true;
    }
}
#[godot_api]
impl IRefCounted for _BaseGEComponent {
    fn get_property(&self, property: StringName) -> Option<Variant> {
        Some(self._get_property(property))
    }

    fn set_property(&mut self, property: StringName, v:Variant) -> bool{
        self._set_property(property, v)
    }
}

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct _BaseGEAccess {
    #[base] base: Base<RefCounted>,
    /// The Flecs component ID for this component.
    flecs_id: EntityId,
    data: *mut [u8],
    world: Gd<_BaseGEWorld>,
}
#[godot_api]
impl _BaseGEAccess {
    fn _component_get(&self, property:StringName) -> Variant {
        let world = self.world.bind();
        let name = world.name_from_flex_id(self.flecs_id);
        let Some(property_data) = world
            .component_definitions
            .get(&name)
            .unwrap()
            .parameters
            .get(&property) else {
                return Variant::nil();
            };
        
        fn get_param<T: ToGodot + Copy>(
            data:*mut [u8],
            property_meta: &ScriptComponetProperty
        ) -> Variant {
            unsafe {
                let param:*mut u8 = &mut (*data)[property_meta.offset];
                let value = *param.cast::<T>();
                return Variant::from(value);
            }
        }
        
        return match property_data.gd_type_id {
            TYPE_BOOL => get_param::<bool>(self.data, property_data),
            TYPE_INT => get_param::<i32>(self.data, property_data),
            TYPE_FLOAT => get_param::<f32>(self.data, property_data),
            _ => todo!(),
        }
    }


    fn _component_set(&mut self, property:StringName, value:Variant) -> bool {
        let world = self.world.bind();
        let name = world.name_from_flex_id(self.flecs_id);
        let Some(property_meta) = world
            .component_definitions
            .get(&name)
            .unwrap()
            .parameters
            .get(&property) else {
                godot_error!(
                    "Can't write to {} in {{component}}. Component has to property with that name",
                    property,
                );
                return false;
            };
        
        fn set_param<T: FromGodot>(
            data:*mut [u8],
            value: Variant,
            property_meta: &ScriptComponetProperty,
        ) {
            unsafe {
                let param:*mut u8 = &mut (*data)[property_meta.offset];
                *param.cast::<T>() = value.to::<T>();
            }
        }
        
        match property_meta.gd_type_id {
            TYPE_BOOL => set_param::<bool>(self.data, value, property_meta),
            TYPE_INT => set_param::<i32>(self.data, value, property_meta),
            TYPE_FLOAT => set_param::<f32>(self.data, value, property_meta),
            _ => todo!(),
        }

        return true;
    }
}
#[godot_api]
impl IRefCounted for _BaseGEAccess {
    fn get_property(&self, property: StringName) -> Option < Variant > {
        Some(self._component_get(property))
    }

    fn set_property(&mut self, property: StringName, value: Variant) -> bool {
        self._component_set(property, value)
    }
}

/// The metadata regarding a component's structure.
#[derive(Debug, Clone)]
struct ScriptComponetDefinition {
    name: StringName,
    parameters: HashMap<StringName, ScriptComponetProperty>,
    flecs_id: EntityId,
    script_id: InstanceId,
    layout: Layout,
}

/// The definition for one property in a component's definition.
#[derive(Debug, Default, Clone)]
struct ScriptComponetProperty {
    name: StringName,
    gd_type_id: i32,
    offset: usize,
}

#[derive(Debug, Clone)]
struct ScriptSystemContext {
    callable: Callable,
    terms: Array<Gd<Script>>,
    /// The arguments passed to the system.
    sysatem_args: Array<Variant>,
    /// Holds the accesses stored in `sysatem_args` for quicker access.
    term_accesses: Box<[Gd<_BaseGEAccess>]>,
    world: Gd<_BaseGEWorld>,
}

#[derive(GodotClass)]
#[class(base=Node)]
struct _BaseGEWorld {
    #[base] node: Base<Node>,
    world: FlWorld,
    component_definitions: ComponentDefinitions,
    system_contexts: LinkedList<Pin<Box<ScriptSystemContext>>>,
}
#[godot_api]
impl _BaseGEWorld {
    #[func]
    fn _world_process(&mut self, delta:f32) {
        self.world.progress(delta);
    }

    /// Defines a new component to be used in the world.
    #[func]
    fn add_component(
        &mut self,
        component_name: StringName,
        mut component: Gd<Script>,
    ) {
        if self.component_definitions.has(component.instance_id()) {
            panic!("Component with that script already registered TODO: better msg")
        }
        if self.component_definitions.has(&component_name) {
            panic!("Component with that name already registered TODO: better msg")
        }

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
                .to::<i32>();
            if property_type == TYPE_NIL {
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


        let layout = Self::layout_from_properties(&component_properties);
        let mut script_component = ScriptComponetDefinition {
            name: component_name.clone(),
            parameters: component_properties,
            flecs_id: 0,
            script_id: component.instance_id(),
            layout,
        };
        script_component.flecs_id = self.world
            .component_dynamic(component_name, layout);
        
        self.component_definitions.insert(script_component);
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
    fn _new_entity(&mut self, with_components:Array<Gd<Script>>) -> Gd<_BaseGEEntity> {
        let mut entity = self.world.entity();
        let mut i = 0;
        while i != with_components.len() {
            let script = with_components.get(i);
            let comp_def = self.component_definitions
                .get(script.instance_id())
                .unwrap();
            entity = entity.add_id(comp_def.flecs_id);
            i += 1;
        }

        Gd::from_init_fn(|base| {
            _BaseGEEntity {
                base,
                world: self.to_gd(),
                id: entity.id(),
            }
        })
    }

    fn name_from_flex_id(&self, flecs_component_id: EntityId) -> StringName {
        self.component_definitions
            .get(flecs_component_id)
            .unwrap()
            .name
            .clone()
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
        let mut tarm_accesses = vec![];
        for term_i in 0..terms.len() as usize {
            let term_script = terms.get(term_i).clone();
            if let Ok(_) = term_script.try_cast::<GdScript>() {
                let mut compopnent_access = Gd
                    ::<_BaseGEAccess>
                    ::from_init_fn(|base| {
                        _BaseGEAccess {
                            base,
                            flecs_id: term_ids[term_i],
                            data: &mut [],
                            world: self.to_gd(),
                        }
                    });
                let comp_access_script = load::<Script>(
                    "res://addons/glecs_godot_plugin/gd/component_access.gd"
                );
                compopnent_access.set_script(comp_access_script.to_variant());
                system_args.push(compopnent_access.to_variant());
                tarm_accesses.push(compopnent_access);
            } else {
                panic!();
            }
        }
        let term_args_fast = tarm_accesses
            .into_boxed_slice();

        // Create contex
        self.system_contexts.push_back(Pin::new(Box::new(
            ScriptSystemContext {
                sysatem_args: system_args,
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
        
        let mut sys = self.world.system()
            .context_ptr(context_ptr.cast::<c_void>());
        for id in term_ids.iter() {
            sys = sys.term_dynamic(*id);
        }
        let system_fn:fn(&Iter) = |iter| {
            let context = unsafe {
                (iter as *const Iter)
                    .cast_mut()
                    .as_mut()
                    .unwrap()
                    .get_context_mut::<Pin<Box<ScriptSystemContext>>>()
            };

            let mut columns:Vec<flecs::ColumnDynamic> = vec![];
            for i in 1..=(iter.field_count()) as i32 {
                let column = iter.field_dynamic(i);
                columns.push(column);
            }

            for entity_i in 0..(iter.count() as usize) {
                // Create components arguments
                for field_i in 0..iter.field_count() as usize {
                    let column = columns
                        .get_mut(field_i)
                        .unwrap();
                    let data:*mut [u8] = column.get_mut(entity_i);

                    context.term_accesses[field_i].bind_mut().data = data;
                }
                
                let _result = context.callable.callv(
                    context.sysatem_args.clone()
                );
            }
        };
        sys.iter(system_fn);
    }

    fn script_is_component(script: Gd<Script>) -> bool {
        todo!()
    }

    fn get_component_description(
        &self,
        key:impl Into<ComponentDefinitionsMapKey>,
    ) -> Option<&ScriptComponetDefinition> {
        self.component_definitions.get(key)
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

    fn physics_process(&mut self, delta:f64) {
        self.world.progress(delta as f32);
    }
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

    fn get_value<'a>(&self, d:&'a ComponentDefinitions) -> Option<&'a ScriptComponetDefinition> {
        d.data.get(self.get_index(d))
    }

    fn get_value_mut<'a>(&self, d:&'a mut ComponentDefinitions) -> Option<&'a mut ScriptComponetDefinition> {
        let idnex = self.get_index(d);
        d.data.get_mut(idnex)
    }
    
    fn set_value(&self, d:&mut ComponentDefinitions, value:ScriptComponetDefinition) {
        let idnex = self.get_index(d);
        d.data[idnex] = value;
    }
} impl From<StringName> for ComponentDefinitionsMapKey {
    fn from(value: StringName) -> Self {
        ComponentDefinitionsMapKey::Name(value)
    }
} impl From<EntityId> for ComponentDefinitionsMapKey {
    fn from(value: EntityId) -> Self {
        ComponentDefinitionsMapKey::FlecsId(value)
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
    data: Vec<ScriptComponetDefinition>,
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

    fn insert(&mut self, element:ScriptComponetDefinition) {
        let len: usize = self.data.len();
        self.add_mapping(
            len,
            element.name.clone(),
            element.flecs_id,
            element.script_id.clone(),
        );
        self.data.push(element);
    }

    fn get(
        &self,
        key:impl Into<ComponentDefinitionsMapKey>,
    ) -> Option<&ScriptComponetDefinition> {
        let x = key.into();
        x.get_value(self)
    }

    fn get_mut(
        &mut self,
        key:impl Into<ComponentDefinitionsMapKey>,
    ) -> Option<&mut ScriptComponetDefinition> {
        let x = key.into();
        x.get_value_mut(self)
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
