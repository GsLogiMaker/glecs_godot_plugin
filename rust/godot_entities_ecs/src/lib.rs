
use std::alloc::Layout;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::mem::size_of;
use std::pin::Pin;
use std::sync::Mutex;

use flecs::EntityId;
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
use godot::engine::NodeVirtual;
use godot::engine::Object;
use godot::engine::ObjectVirtual;
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

struct SuperECGDS; #[gdextension] unsafe impl ExtensionLibrary for SuperECGDS {}

#[derive(GodotClass)]
#[class(base=RefCounted)]
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
    #[func]
    fn _component_get(&self, property:StringName) -> Variant {
        let world = unsafe {&*self.world};
        let name = world.name_from_flex_id(self.flecs_id);
        let Some(property_data) = world.script_components
            .get(&name).unwrap().parameters.get(&property) else {
                return Variant::nil();
            };
        
        fn get_param<T: ToVariant + Copy>(
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
    fn _component_set(&mut self, property:StringName, value:Variant) -> bool {
        assert!(self.mutable);
        let world = unsafe {&*self.world};
        let name = world.name_from_flex_id(self.flecs_id);
        let Some(property_meta) = world.script_components
            .get(&name).unwrap().parameters.get(&property) else {
                return false;
            };
        
        fn set_param<T: FromVariant>(
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

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct _BaseGEComponentAccess {
    #[base] base: Base<RefCounted>,
    /// The Flecs component ID for this component.
    flecs_id: EntityId,
    data: *mut [u8],
    world: *const _BaseGEWorld,
    mutable: bool,
}
#[godot_api]
impl _BaseGEComponentAccess {
    #[func]
    fn _component_get(&self, property:StringName) -> Variant {
        let world = unsafe {&*self.world};
        let name = world.name_from_flex_id(self.flecs_id);
        let Some(property_data) = world.script_components
            .get(&name).unwrap().parameters.get(&property) else {
                return Variant::nil();
            };
        
        fn get_param<T: ToVariant + Copy>(
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
    fn _component_set(&mut self, property:StringName, value:Variant) -> GdError {
        if !self.mutable {
            return GdError::ERR_DATABASE_CANT_WRITE;
        }
        let world = unsafe {&*self.world};
        let name = world.name_from_flex_id(self.flecs_id);
        let Some(property_meta) = world.script_components
            .get(&name).unwrap().parameters.get(&property) else {
                return GdError::ERR_DOES_NOT_EXIST;
            };
        
        fn set_param<T: FromVariant>(
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

        return GdError::OK;
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
    world: *const _BaseGEWorld,
}

#[derive(GodotClass)]
#[class(base=RefCounted)]
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

    #[func]
    fn _term(&mut self, component: Gd<Script>) {
        self.terms.push((component, false));
    }

    #[func]
    fn _mut_term(& mut self, component: Gd<Script>) {
        self.terms.push((component, true));
    }

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
        godot_print!("_world_process {delta}");
        self.world.progress(delta);
    }

    #[func]
    fn _new_component(
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

        self.component_names
            .insert(component.instance_id(), component_name.clone());

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
        godot_print!("Registered component: {:?}", component_name);
    }
    
    fn _new_system_from_builder(&mut self, builder: &_BaseGESystemBuilder) {
        let terms = &builder.terms;

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

        let world_ptr:*const _BaseGEWorld = self;
        self.system_contexts.push_back(Pin::new(Box::new(
            ScriptSystemContext {
                callable: builder.callable.clone(),
                terms: terms.iter()
                    .map(|x| { (x.0.clone(), x.1) })
                    .collect(),
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

        sys.iter(|iter| {
            let context = unsafe {
                iter.get_context::<Pin<Box<ScriptSystemContext>>>()
            };
            let world = context.world;
            let terms = &context.terms;
            let callable = &context.callable;

            let mut columns:Vec<flecs::ColumnDynamic> = vec![];
            for i in 1..=(iter.field_count()) as i32 {
                let column = iter.field_dynamic(i);
                columns.push(column);
            }

            for entity_i in 0..(iter.count() as usize) {
                // Create components arguments
                let mut args: Array<Variant> = array![];
                args.resize(iter.field_count() as usize);
                let fields = iter.raw_fields();
                for field_i in 0..iter.field_count() as usize {
                    let column = columns
                        .get_mut(field_i)
                        .unwrap();
                    let (term_script, term_mutable)
                        = terms[field_i].clone();
                    let data:*mut [u8] = if term_mutable {
                        column.get_mut(entity_i)
                    } else {
                        let data_ptr:*const[u8] = column.get(field_i);
                        unsafe {
                            // `[_BaseECSComponent]` makes sure this reference
                            // is not mutated.
                            data_ptr.cast_mut().as_mut().unwrap()
                        }
                    };

                    if let Some(term_script) = term_script.try_cast::<GdScript>() {
                        let mut compopnent_access = Gd::<_BaseGEComponentAccess>::with_base(|base| {
                            _BaseGEComponentAccess {
                                base,
                                flecs_id: fields.get(field_i).unwrap().id,
                                data,
                                world,
                                mutable: term_mutable,
                            }
                        });
                        let comp_access_script = load::<Script>("res://addons/g_ecs/src/component_access.gd");
                        // compopnent_access.set_script(term_script.to_variant());
                        compopnent_access.set_script(comp_access_script.to_variant());
                        args.set(field_i, compopnent_access.to_variant());
                    } else {
                        panic!();
                    }
                }
                
                let _result = callable.callv(args);
            }
        });
    }


    #[func]
    fn _new_entity(&mut self, with_components:Array<Gd<Script>>) {
        let mut entity = self.world.entity();
        let mut i = 0;
        while i != with_components.len() {
            let script = with_components.get(i);
            let component_name = self.component_names
                .get(&script.instance_id())
                .unwrap();
            let script_component = self.script_components
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
#[godot_api]
impl NodeVirtual for _BaseGEWorld {
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

#[derive(GodotClass)]
#[class(base=Object)]
struct Entity {
    #[base] base: Base<Object>,
    entity: FlEntity,
}
#[godot_api]
impl Entity {
    fn set_component(&mut self, components: Array<StringName>) {
        let world = Engine::singleton()
            .get_singleton(StringName::from("ECSWorld"))
            .unwrap()
            .cast::<_BaseGEWorld>();
        let script = world.get_script().to::<Gd<Script>>();
        // script.get_class()
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
