
use std::alloc::Layout;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::mem::size_of;
use std::pin::Pin;
use std::sync::Mutex;

use flecs::EntityId;
use flecs::Iter;
use flecs::ecs_set_threads;
use flecs::world::World as FlWorld;
use flecs::Entity as FlEntity;
use flecs::TermBuilder;

use godot::engine;
use godot::engine::GdScript;
use godot::engine::Script;
use godot::obj::EngineEnum;
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
    /* PACKED_BYTE_ARRAY */ size_of::<Array<()>>(),
    /* PACKED_INT32_ARRAY */ size_of::<Array<()>>(),
    /* PACKED_INT64_ARRAY */ size_of::<Array<()>>(),
    /* PACKED_FLOAT32_ARRAY */ size_of::<Array<()>>(),
    /* PACKED_FLOAT64_ARRAY */ size_of::<Array<()>>(),
    /* PACKED_STRING_ARRAY */ size_of::<Array<()>>(),
    /* PACKED_VECTOR2_ARRAY */ size_of::<Array<()>>(),
    /* PACKED_VECTOR3_ARRAY */ size_of::<Array<()>>(),
    /* PACKED_COLOR_ARRAY */ size_of::<Array<()>>(),
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
    fn set_component(&mut self, component:Gd<Script>, value:Variant) {
        let world = self.world.bind_mut();
        let component_name = world
            .component_names
            .get(&component.instance_id())
            .ok_or_else(|| { format!(
                "Can't set component '{}' in entity. That component hasn't been registered with the world.",
                component,
            )})
            .unwrap();
		// let component_id = unsafe { ecs_get_mut_id(self.world, self.entity, comp_id) };
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
    /// The Flecs component ID for this component.
    flecs_id: EntityId,
    data: *mut [u8],
    world: *const _BaseGEWorld,
    mutable: bool,
}
#[godot_api]
impl _BaseGEComponent {
}

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct _BaseGEAccess {
    #[base] base: Base<RefCounted>,
    /// The Flecs component ID for this component.
    flecs_id: EntityId,
    data: *mut [u8],
    world: *const _BaseGEWorld,
    mutable: bool,
}
#[godot_api]
impl _BaseGEAccess {
    #[func]
    fn _component_get(&self, property:StringName) -> Variant {
        let world = unsafe {&*self.world};
        let name = world.name_from_flex_id(self.flecs_id);
        let Some(property_data) = world.script_components
            .get(&name).unwrap().parameters.get(&property) else {
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
        
        match property_data.type_id {
            TYPE_BOOL => {
                return get_param::<bool>(self.data, property_data);
            }
            TYPE_INT => {
                return get_param::<i32>(self.data, property_data);
            }
            TYPE_FLOAT => {
                return get_param::<f32>(self.data, property_data);
            }
            _ => {
                todo!()
            }
        }
    }

    #[func]
    fn _component_has(&self, property:StringName) -> bool {
        let world = unsafe {&*self.world};
        let name = world.name_from_flex_id(self.flecs_id);
        let Some(property_data) = world.script_components
            .get(&name).unwrap().parameters.get(&property) else {
                return false;
            };
        
        return true;
    }

    #[func]
    fn _component_set(&mut self, property:StringName, value:Variant) -> bool {
        if !self.mutable {
            godot_error!(
                "Can't write to {} in {{component}}. The component was queried as read only.",
                property,
            );
            return false;
        }
        let world = unsafe {&*self.world};
        let name = world.name_from_flex_id(self.flecs_id);
        let Some(property_meta) = world.script_components
            .get(&name).unwrap().parameters.get(&property) else {
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
        
        match property_meta.type_id {
            TYPE_BOOL => {
                set_param::<bool>(self.data, value, property_meta);
            }
            TYPE_INT => {
                set_param::<i32>(self.data, value, property_meta);
            }
            TYPE_FLOAT => {
                set_param::<f32>(self.data, value, property_meta);
            } _ => {
                todo!()
            }
        }

        return true;
    }

    #[func]
    fn _component_get_property_list(&self) -> Array<Dictionary> {
        return array![];
    }
}


#[derive(Debug, Default, Clone)]
struct ScriptComponetMetadata {
    name: StringName,
    parameters: HashMap<StringName, ScriptComponetProperty>,
    flecs_id: EntityId,
}

#[derive(Debug, Default, Clone)]
struct ScriptComponetProperty {
    name: StringName,
    type_id: i32,
    offset: usize,
}

#[derive(Debug, Clone)]
struct ScriptSystemContext {
    callable: Callable,
    terms: Vec<(Gd<Script>, bool)>,
    /// The arguments passed to the system.
    sysatem_args: Array<Variant>,
    /// Holds the accesses stored in `sysatem_args` for quicker access.
    term_accesses: Box<[Gd<_BaseGEAccess>]>,
    world: *const _BaseGEWorld,
}

#[derive(GodotClass)]
#[class(base=RefCounted)]
/// Builds a new ECS system.
struct _BaseGESystemBuilder {
    callable: Callable,
    terms: Vec<(Gd<Script>, bool)>,
    #[export] _world: Gd<_BaseGEWorld>,
}
#[godot_api]
impl _BaseGESystemBuilder {
    #[func]
    fn _new_for_world(
        callable: Callable,
        world: Gd<_BaseGEWorld>,
    ) -> Gd<_BaseGESystemBuilder> {
        Gd::new(_BaseGESystemBuilder {
            callable,
            terms: vec![],
            _world: world,
        })
    }

    /// Adds a readable term to the system's query.
    #[func]
    fn _reads(&mut self, component: Gd<Script>) {
        self.terms.push((component, false));
    }

    /// Adds a writable term to the system's query.
    #[func]
    fn _writes(& mut self, component: Gd<Script>) {
        self.terms.push((component, true));
    }

    /// Finalizes the builder and adds it to the world.
    #[func]
    fn _build(system: Gd<_BaseGESystemBuilder>, mut world: Gd<_BaseGEWorld>) {
        let mut world = world.bind_mut();
        world._new_system_from_builder(&*system.bind());
    }
}

#[derive(GodotClass)]
#[class(base=Node)]
struct _BaseGEWorld {
    #[base] node: Base<Node>,
    world: FlWorld,
    /// Maps component scripts to their component names. Keys are the
    /// `[InstanceId]`s of the `[Script]`s.
    component_names: HashMap<InstanceId, StringName>,
    /// 
    script_components: HashMap<StringName, ScriptComponetMetadata>,
    component_ids_to_names: HashMap<EntityId, StringName>,
    system_contexts: LinkedList<Pin<Box<ScriptSystemContext>>>,
}
#[godot_api]
impl _BaseGEWorld {
    #[func]
    fn _world_process(&mut self, delta:f32) {
        self.world.progress(delta);
    }

    /// Creates a new component in the world.
    #[func]
    fn new_component(
        &mut self,
        component_name: StringName,
        mut component: Gd<Script>,
    ) {
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
                    type_id: property_type,
                    offset,
                },
            );

            offset += TYPE_SIZES[property_type as usize];
            i += 1;
        }

        let mut script_component = ScriptComponetMetadata {
            name: component_name.clone(),
            parameters: component_properties,
            flecs_id: 0,
        };
        let layout = Self::layout_from_script_component(&script_component);

        if !self.component_names.contains_key(&component.instance_id()) {
            self.component_names
                .insert(component.instance_id(), component_name.clone());
        }

        script_component.flecs_id = unsafe {
            // This unsafe block converts component_name into &'static str to
            // be passed into the symbol parameter.
            // The 'component_dynamic' parameter only converts symbol to an
            // owned string, so it is ok to extend it's lifetime to 'static.
            let string = component_name.to_string();
            let str:*const str = string.as_str();
            let str = str.as_ref::<'static>().unwrap();
            let flecs_id = self.world.component_dynamic(str, layout);
            flecs_id
        };
        self.component_ids_to_names.insert(
            script_component.flecs_id,
            component_name.clone(),
        );
        self.script_components.insert(
            component_name.clone(),
            script_component,
        );
    }

    /// Creates a new entity in the world.
    #[func]
    fn new_entity(&mut self, with_components:Array<Gd<Script>>) {
        let mut entity = self.world.entity();
        let mut i = 0;
        while i != with_components.len() {
            let script = with_components.get(i);
            let component_name = self
                .component_names
                .get(&script.instance_id())
                .unwrap();
            let script_component = self
                .script_components
                .get(component_name)
                .unwrap();
            entity = entity.add_id(script_component.flecs_id);
            i += 1;
        }
    }

    fn name_from_flex_id(&self, flecs_component_id: EntityId) -> &StringName {
        let names = &self.component_ids_to_names;
        names.get(&flecs_component_id)
            .unwrap()
    }

    fn _new_system_from_builder(&mut self, builder: &_BaseGESystemBuilder) {
        let terms = &builder.terms;

        // Create term list
        let mut term_ids = vec![];
        for i in 0..terms.len() {
            let (script, mutable) = terms.get(i).unwrap();
            let name = self.component_names
                .get(&script.instance_id())
                .unwrap();
            let script_component = self.script_components
                .get(name)
                .unwrap();
            term_ids.push((script_component.flecs_id, mutable));
        }

        // Create component accesses
        let terms_copy:Vec<(Gd<Script>, bool)> = terms.iter()
            .map(|x| { (x.0.clone(), x.1) })
            .collect();
        let mut system_args = array![];
        let mut tarm_accesses = vec![];
        let world_ptr:*const _BaseGEWorld = self;
        for term_i in 0..terms.len() as usize {
            let (term_script, term_mutable) = terms[term_i]
                .clone();
            if let Ok(_) = term_script.try_cast::<GdScript>() {
                let mut compopnent_access = Gd
                    ::<_BaseGEAccess>
                    ::with_base(|base| {
                        _BaseGEAccess {
                            base,
                            flecs_id: term_ids[term_i].0,
                            data: &mut [],
                            world: world_ptr,
                            mutable: term_mutable,
                        }
                    });
                let comp_access_script = load::<Script>(
                    "res://addons/g_ecs/src/component_access.gd"
                );
                compopnent_access.set_script(comp_access_script.to_variant());
                system_args.push(compopnent_access.to_variant());
                tarm_accesses.push(compopnent_access);
            } else {
                panic!();
            }
        }
        let term_args_fast = tarm_accesses.into_boxed_slice();

        // Create contex
        self.system_contexts.push_back(Pin::new(Box::new(
            ScriptSystemContext {
                sysatem_args: system_args,
                term_accesses: term_args_fast,
                callable: builder.callable.clone(),
                terms: terms_copy,
                world: world_ptr,
            }
        )));
        let context_ptr:*mut Pin<Box<ScriptSystemContext>> = self.system_contexts
            .back_mut()
            .unwrap();
        
        let mut sys = self.world.system()
            .context_ptr(context_ptr.cast::<c_void>());
        for id in term_ids.iter() {
            sys = sys.term_dynamic(id.0);
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
                    let term_mutable = context.terms[field_i].1;
                    let data:*mut [u8] = match term_mutable {
                        // Get mutable component data
                        true => {column.get_mut(entity_i)},
                        // Get immutable component data
                        false => {
                            // `[_BaseECSComponent]` makes sure this
                            // reference is not mutated.
                            (column.get(field_i) as *const [u8])
                                .cast_mut()
                        },
                    };

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

    fn layout_from_script_component(component: &ScriptComponetMetadata) -> Layout {
        let mut size = 0;
        for (_name, property) in &component.parameters {
            size += TYPE_SIZES[property.type_id as usize];
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
            component_names: HashMap::default(),
            script_components: HashMap::default(),
            component_ids_to_names: HashMap::default(),
            system_contexts: LinkedList::default(),
        }
    }

    fn physics_process(&mut self, delta:f64) {
        self.world.progress(delta as f32);
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
