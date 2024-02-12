
use std::alloc::Layout;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::ffi::c_void;
use std::hash::Hash;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::rc::Rc;

use flecs::EntityId;
use flecs::Iter;
use flecs::TermBuilder;
use flecs::World as FlWorld;
use godot::engine::notify::NodeNotification;
use godot::engine::Script;
use godot::prelude::*;

use crate::component::_BaseGEComponent;
use crate::component_definitions::ComponentDefinitions;
use crate::component_definitions::ComponentDefinitionsMapKey;
use crate::component_definitions::ComponetDefinition;
use crate::component_definitions::ComponetProperty;
use crate::entity::EntityLike;
use crate::entity::_BaseGEEntity;
use crate::prefab::PrefabDefinition;
use crate::prefab::PREFAB_COMPONENTS;
use crate::show_error;
use crate::TYPE_SIZES;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct _BaseGEWorld {
    #[base] pub(crate) node: Base<Node>,
    pub(crate) world: FlWorld,
    component_definitions: ComponentDefinitions,
    system_contexts: LinkedList<Pin<Box<ScriptSystemContext>>>,
    gd_entity_map: HashMap<EntityId, Gd<_BaseGEEntity>>,
    prefabs: HashMap<Gd<Script>, Rc<PrefabDefinition>>,
    /// Maps identifiers to entities that serve as tags, relations
    /// or other things.
    tag_entities: HashMap<VariantKey, EntityId>,
    pipelines: HashMap<VariantKey, Rc<PipelineDefinition>>,
	deleting:bool
}
#[godot_api]
impl _BaseGEWorld {
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

    #[func]
    fn _entity_from_flecs_id(&mut self, flecs_id:EntityId) -> Gd<_BaseGEEntity> {
        let gd_entity = Gd::from_init_fn(|base| {
            _BaseGEEntity {
                base,
                world: self.to_gd(),
                id: flecs_id,
				world_deletion: false,
                gd_components_map: Default::default(),
            }
        });
        self.gd_entity_map.insert(flecs_id, gd_entity.clone());

        gd_entity
    }


    /// Creates a new entity in the world.
    #[func]
    fn _new_entity(
        &mut self,
        name: String,
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
				world_deletion: false,
                gd_components_map: Default::default(),
            }
        });
        gd_entity.set_name_by_ref(name, self);
        self.gd_entity_map.insert(entity.id(), gd_entity.clone());
        
        gd_entity
    }

    /// Creates a new entity in the world.
    #[func]
    fn new_entity_with_prefab(
        &mut self,
        name:String,
        prefab:Gd<Script>,
    ) -> Gd<_BaseGEEntity> {
        let gd_entity = self._new_entity(name, Array::default());
        let e_id = gd_entity.bind().get_flecs_id();

        let prefab_def = self
            .get_or_add_prefab_definition(prefab);

        unsafe { flecs::ecs_add_id(
            self.world.raw(),
            e_id,
            flecs::ecs_pair(flecs::EcsIsA, prefab_def.flecs_id),
        ) };

        gd_entity
    }


    #[func]
    fn _new_event_listener(
        &mut self,
        event: Variant,
        components: Array<Gd<Script>>,
        callable: Callable,
    ) {
        let mut events:[EntityId;8] = Default::default();
        events[0] = self.get_or_add_tag_entity(event.clone());

        // Make terms list
        let terms = components
            .iter_shared()
            .map(|component|
                self.get_or_add_component(&component).flecs_id
            )
            .collect::<Vec<EntityId>>();
        
        // Allocate context
        let context = Box::new(ScriptSystemContext::new(
            callable.clone(),
            self,
            components,
            Default::default(),
        ));

        // Build filter
        let mut filter_b = self.world.filter_builder();
        for component_id in terms {
            filter_b = filter_b.term_dynamic(component_id);
        }

        // Create observer definition
        let observer_definition = flecs::ecs_observer_desc_t {
            events,
            filter: filter_b.take_filter_desc(),
            callback: Some(Self::raw_system_iteration),
            ctx: (
                Box::leak(context) as *mut ScriptSystemContext
            ).cast::<c_void>(),
            ctx_free: Some(Self::raw_system_drop),
            ..unsafe { MaybeUninit::zeroed().assume_init() }
        };

        // Initialize the observer
        unsafe { flecs::ecs_observer_init(
            self.world.raw(),
            &observer_definition,
        ) };
    }

    #[func]
    fn _new_pipeline(
        &mut self,
        identifier:Variant,
        extra_parameters: Array<Callable>,
    ) {
        self.new_pipeline(identifier, extra_parameters);
    }

    /// Runs all processess associated with the given pipeline.
    #[func]
    fn run_pipeline(&self, pipeline:Variant, delta:f32) {
        let raw_world = self.world.raw();
        let Some(pipeline_id) = self
            .get_pipeline(pipeline.clone())
            else {
                show_error!(
                    "Failed to run pipeline",
                    "No pipeline with identifier {} was defined.",
                    pipeline,
                );
                return;
            };
        unsafe {
            flecs::ecs_set_pipeline(raw_world, pipeline_id.flecs_id);
            flecs::ecs_progress(raw_world, delta);
        }
    }

    // Defines a new system to be run in the world.
    #[func]
    fn _add_system(
        &mut self,
        terms: Array<Gd<Script>>,
        callable: Callable,
        pipeline: Variant,
    ) {
        let nil_id = self.get_or_add_tag_entity(Variant::nil());

        let Some(pipeline_def) = self.get_pipeline(pipeline.clone())
            else {
                show_error!(
                    "Failed to add system",
                    "Noo pipeline with identifer {} was defined.",
                    pipeline,
                );
                return
            };

        // Create term list
        let mut term_ids = vec![];
        for i in 0..terms.len() {
            let term_script = terms.get(i);

            let flecs_id = self.get_id_of_script(term_script.clone())
                .unwrap_or_else(
                    || self.get_or_add_component(&term_script).flecs_id
                );
			
            term_ids.push(flecs_id);
        }

        // Create value getters list
        let mut additional_arg_getters = Vec::new();
        for callable in pipeline_def.extra_parameters.iter_shared() {
            additional_arg_getters.push(callable);
        }
        let value_getters = additional_arg_getters.into_boxed_slice();

        let mut system_args = array![];
        for _v in value_getters.iter() {
            system_args.push(Variant::nil());
        }

        // Create contex
        let context = Pin::new(Box::new(
            ScriptSystemContext::new(
                callable.clone(),
                self,
                terms,
                value_getters,
            )
        ));
        self.system_contexts.push_back(context);
        let context_ptr:*mut Pin<Box<ScriptSystemContext>> = self
            .system_contexts
            .back_mut()
            .unwrap();

        // Create system
        let mut sys = self.world
            .system()
            .context_ptr(context_ptr as *mut c_void);
        for id in term_ids.iter() {
            sys = sys.term_dynamic(*id);
        }

        // System body
        let sys_id = sys.iter(
            Self::system_iteration,
            nil_id,
        ).id();
        unsafe { flecs::ecs_add_id(
            self.world.raw(),
            sys_id,
            pipeline_def.flecs_id,
        ) };
    }

    #[func]
    fn _new_process_system(
        &mut self,
        terms: Array<Gd<Script>>,
        callable: Callable,
    ) {
        let process_identifier = "process".to_variant();
        
        match
            self.get_pipeline(process_identifier.clone())
        {
            Some(_) => {},
            None => {
                // Initialize process pipeline
                let get_process_delta_time = Callable::from_object_method(
                    &self.to_gd(),
                    "get_process_delta_time",
                );
        
                self.new_pipeline(
                    StringName::from("process").to_variant(),
                    Array::from(&[get_process_delta_time]),
                );
            },
        };
        
        self._add_system(terms, callable, process_identifier);
    }

    #[func]
    fn update_frame(&self) {
        self.world.progress(1.0);
    }

    pub(crate) fn get_component_description(
        &self,
        key:impl Into<ComponentDefinitionsMapKey>,
    ) -> Option<Rc<ComponetDefinition>> {
        self.component_definitions.get(key)
    }

    pub(crate) fn get_or_add_component(
        &mut self,
        key: &Gd<Script>,
    ) -> Rc<ComponetDefinition> {
        let value = ComponentDefinitionsMapKey
            ::from(key)
            .get_value(&self.component_definitions);
        match value {
            Some(value) => value,
            None => {
                let def = ComponetDefinition::new(
                    key.clone(),
                    self,
                );
                self.component_definitions.insert(def)
            }
        }
    }


    pub(crate) fn new_pipeline(
        &mut self,
        identifier:Variant,
        extra_parameters: Array<Callable>,
    ) -> Rc<PipelineDefinition> {
        let key = VariantKey {variant: identifier};
        if let Some(def) = self.pipelines.get(&key) {
            return def.clone()
        }

        // Initialize pipeline
        let pipeline_id = unsafe { flecs::ecs_new_id(self.world.raw()) };
            
        let mut system_query = flecs::ecs_query_desc_t{
            ..unsafe { MaybeUninit::zeroed().assume_init() }
        };
        system_query.filter.terms[0] = flecs::ecs_term_t {
            id: pipeline_id,
            ..unsafe { MaybeUninit::zeroed().assume_init() }
        };

        unsafe { flecs::ecs_pipeline_init(
            self.world.raw(),
            &flecs::ecs_pipeline_desc_t {
                entity: pipeline_id,
                query: system_query,
            },
        ) };

        let def = PipelineDefinition {
            extra_parameters,
            flecs_id: pipeline_id,
        };
        self.pipelines.insert(key.clone(), Rc::new(def));

        self.pipelines.get(&key).unwrap().clone()
    }

    pub(crate) fn get_pipeline(
        &self,
        identifier:Variant,
    ) -> Option<Rc<PipelineDefinition>> {
        let key = VariantKey {variant: identifier};
        self.pipelines.get(&key).map(|x| x.clone())
    }

    fn get_or_add_prefab_definition(&mut self, script:Gd<Script>) -> Rc<PrefabDefinition> {
        if let Some(prefab_def) = self.prefabs.get(&script) {
            return prefab_def.clone()
        }
        let prefab = self
            .new_prefab_def(script.clone());
        self.prefabs.insert(script.clone(), prefab);
        self.prefabs.get(&script).unwrap().clone()
    }

    pub(crate) fn get_or_add_tag_entity(&mut self, key:Variant) -> EntityId {
        if let Ok(entity_gd) = key.try_to::<Gd<_BaseGEEntity>>() {
            return entity_gd.bind().get_flecs_id()
        }

        let key = VariantKey {variant: key};
        
        self.tag_entities.get(&key)
            .map(|x| *x)
            .unwrap_or_else(|| {
                let id = self.world.entity().id();
                self.tag_entities.insert(key, id);
                id
            })
    }

    pub(crate) fn has_tag_entity(&mut self, key:Variant) -> bool {
        if let Ok(entity_gd) = key.try_to::<Gd<_BaseGEEntity>>() {
            return true
        }

        let key = VariantKey {variant: key};
        self.tag_entities.contains_key(&key)
    }

    pub(crate) fn layout_from_properties(
        parameters: &HashMap<StringName, ComponetProperty>,
    ) -> Layout {
        let mut size = 0;
        for (_name, property) in parameters {
            size += TYPE_SIZES[property.gd_type_id as usize];
        }
        Layout::from_size_align(size, 8).unwrap()
    }

    fn get_id_of_script(&self, mut script: Gd<Script>) -> Option<EntityId> {
        let Some(x) = self.component_definitions
            .get(&script) 
            else {
                return None
            };
        Some(x.flecs_id)
    }

	pub(crate) fn on_entity_freed(&mut self, entity_id:EntityId) {
		if self.deleting {
			return;
		}
		self.gd_entity_map.remove(&entity_id);
	}

	pub(crate) fn on_free(&mut self) {
		self.deleting = true;
		for (_, gd_entity) in &mut self.gd_entity_map {
			gd_entity.bind_mut().world_deletion = true;
			gd_entity.clone().free();
		}
	}

	pub(crate) fn system_iteration(iter:&Iter) {
		let context = unsafe {
			(iter as *const Iter)
				.cast_mut()
				.as_mut()
				.unwrap()
				.get_context_mut::<Pin<Box<ScriptSystemContext>>>()
		};

        // Update extra variables
        let mut system_args_ref = context.system_args.clone();
        for (i, getter) in context.additional_arg_getters.iter().enumerate() {
            system_args_ref.set(i, getter.callv(Array::default()));
        }

		for entity_index in 0..(iter.count() as usize) {
			// Create components arguments
			for field_i in 0i32..(iter.field_count()) {
				let mut column = iter
					.field_dynamic(field_i+1);
				let data:*mut [u8] = column.get_mut(entity_index);

				context.term_accesses[field_i as usize]
					.bind_mut()
					.data = data;
			}

			let _result = context.callable.callv(
				context.system_args.clone()
			);
		}
	}

    

    pub(crate) fn new_prefab_def(
        &mut self,
        mut script:Gd<Script>,
    ) -> Rc<PrefabDefinition> {
        let prefab_entt = self.world
            .prefab(&script.instance_id().to_string());

        let componets = script.get_script_constant_map()
            .get(StringName::from(PREFAB_COMPONENTS))
            .unwrap_or_else(|| {Array::<Variant>::default().to_variant()})
            .try_to::<Array<Variant>>()
            .unwrap_or_default();

        for component in componets.iter_shared() {
            let Ok(component) = component.try_to::<Gd<Script>>()
                else {continue};
                
            prefab_entt.add_id(
                self.get_or_add_component(&component).flecs_id
            );
        }

        Rc::new(PrefabDefinition {
            script: script,
            flecs_id: prefab_entt.id(),
        })
    }


    extern "C" fn raw_system_iteration(iter:*mut flecs::ecs_iter_t) {
        let mut iter = Iter::new(iter);

		let context = unsafe {
            // Here we decouple the mutable reference of the context
            // from the rest of Iter.
			(
                iter.get_context_mut::<ScriptSystemContext>()
                    as *mut ScriptSystemContext
            )
                .as_mut()
                .unwrap()
		};

        // Update extra variables
        let mut system_args_ref = context.system_args.clone();
        for (i, getter) in
            context.additional_arg_getters.iter().enumerate()
        {
            system_args_ref.set(i, getter.callv(Array::default()));
        }

		for entity_index in 0..(iter.count() as usize) {
			// Create components arguments
			for field_i in 0i32..(iter.field_count()) {
				let mut column = iter
					.field_dynamic(field_i+1);
				let data:*mut [u8] = column.get_mut(entity_index);

				context.term_accesses[field_i as usize]
					.bind_mut()
					.data = data;
			}
			
			let _result = context.callable.callv(
				context.system_args.clone()
			);
		}
	}

    extern "C" fn raw_system_drop(void_ptr:*mut c_void) {
        let ptr = void_ptr
            .cast::<ScriptSystemContext>();
        let boxed = unsafe { Box::from_raw(ptr) };
        drop(boxed)
	}
}

#[godot_api]
impl INode for _BaseGEWorld {
    fn init(node: Base<Node>) -> Self {
        let world = FlWorld::new();
        let mut gd_world = Self {
            node,
            world: world,
            component_definitions: Default::default(),
            system_contexts: Default::default(),
            gd_entity_map: Default::default(),
            prefabs: Default::default(),
            tag_entities: Default::default(),
            pipelines: Default::default(),
			deleting: false,
        };

        gd_world.tag_entities.insert(
            StringName::from("on_add").to_variant().into(),
            unsafe { flecs::EcsOnAdd },
        );
        gd_world.tag_entities.insert(
            StringName::from("on_remove").to_variant().into(),
            unsafe { flecs::EcsOnRemove },
        );
        gd_world.tag_entities.insert(
            StringName::from("on_set").to_variant().into(),
            unsafe { flecs::EcsOnSet },
        );
        gd_world.tag_entities.insert(
            StringName::from("on_unset").to_variant().into(),
            unsafe { flecs::EcsUnSet },
        );
        gd_world.tag_entities.insert(
            StringName::from("on_monitor").to_variant().into(),
            unsafe { flecs::EcsMonitor },
        );
        gd_world.tag_entities.insert(
            StringName::from("on_delete").to_variant().into(),
            unsafe { flecs::EcsOnDelete },
        );
        gd_world.tag_entities.insert(
            StringName::from("on_table_create").to_variant().into(),
            unsafe { flecs::EcsOnTableCreate },
        );
        gd_world.tag_entities.insert(
            StringName::from("on_table_delete").to_variant().into(),
            unsafe { flecs::EcsOnTableDelete },
        );
        gd_world.tag_entities.insert(
            StringName::from("on_table_empty").to_variant().into(),
            unsafe { flecs::EcsOnTableEmpty },
        );
        gd_world.tag_entities.insert(
            StringName::from("on_table_fill").to_variant().into(),
            unsafe { flecs::EcsOnTableFill },
        );

        gd_world
    }

    fn ready(&mut self) {
        let get_process_delta_time = Callable::from_object_method(
            &self.to_gd(),
            "get_process_delta_time",
        );
        let get_physics_process_delta_time = Callable::from_object_method(
            &self.to_gd(),
            "get_physics_process_delta_time",
        );

        self.new_pipeline(
            StringName::from("process").to_variant(),
            Array::from(&[get_process_delta_time]),
        );
        self.new_pipeline(
            StringName::from("physics_process").to_variant(),
            Array::from(&[get_physics_process_delta_time]),
        );
    }

	fn on_notification(&mut self, what: NodeNotification) {
        match what {
            NodeNotification::Predelete => {
                self.on_free()
            },
            _ => {},
        }
    }

    fn process(&mut self, delta:f64) {
        self.run_pipeline("process".to_variant(), delta as f32);
    }

    fn physics_process(&mut self, delta:f64) {
        self.run_pipeline("physics_process".to_variant(), delta as f32);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ScriptSystemContext {
    callable: Callable,
    terms: Array<Gd<Script>>,
    /// The arguments passed to the system.
    system_args: Array<Variant>,
    /// Holds the accesses stored in `sysatem_args` for quicker access.
    term_accesses: Box<[Gd<_BaseGEComponent>]>,
    world: Gd<_BaseGEWorld>,
    /// A list of getters for extra arguments in a pipeline.
    additional_arg_getters: Box<[Callable]>,
} impl ScriptSystemContext {
    fn new(
        callable: Callable,
        world: &mut _BaseGEWorld,
        terms: Array<Gd<Script>>,
        additional_arg_getters: Box<[Callable]>,
    ) -> Self {
        let component_class_name = <_BaseGEComponent as GodotClass>
            ::class_name()
            .to_string_name();

        // Make arguments list
        let mut args = array![];
        for _v in additional_arg_getters.iter() {
            args.push(Variant::nil());
        }

        // Create component accesses
        let mut tarm_accesses: Vec<Gd<_BaseGEComponent>> = vec![];
        for term_i in 0..terms.len() as usize {
            let term_script = terms.get(term_i);
            let script_type = term_script.get_instance_base_type();
            if script_type != component_class_name {
                continue
            }
            let mut compopnent_access = Gd
                ::<_BaseGEComponent>
                ::from_init_fn(|base| {
                    _BaseGEComponent {
                        base,
                        data: &mut [],
                        component_definition: world
                            .get_or_add_component(&term_script),
                    }
                });
            compopnent_access.set_script(term_script.to_variant());
            args.push(compopnent_access.to_variant());
            tarm_accesses.push(compopnent_access);
        }
        let term_args_fast = tarm_accesses
            .into_boxed_slice();

        Self {
            callable: callable,
            terms: terms,
            system_args: args,
            term_accesses: term_args_fast,
            world: world.to_gd(),
            additional_arg_getters,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PipelineDefinition {
    extra_parameters: Array<Callable>,
    flecs_id: EntityId,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct VariantKey {
    variant: Variant
} impl Eq for VariantKey {
} impl Hash for VariantKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Variant::hash(&self.variant).hash(state);
    }
} impl From<Variant> for VariantKey {
    fn from(value: Variant) -> Self {
        VariantKey { variant: value }
    }
}