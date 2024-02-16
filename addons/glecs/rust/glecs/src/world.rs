
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
use flecs::SystemBuilder;
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
use crate::queries;
use crate::queries::BuildType;
use crate::queries::_BaseSystemBuilder;
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
            
            // Initialize component properties
            // TODO: Initialize properties in deterministic order
            for property in comp_def.parameters.iter() {
                // TODO: Get default values of properties
                let default_value = script
                    .get_property_default_value(property.name.clone());
                _BaseGEComponent::_initialize_property(
                    data,
                    comp_def.as_ref(),
                    property.name.clone(),
                    default_value,
                );
            }

            i += 1;
        }

        let mut gd_entity = Gd::from_init_fn(|base| {
            _BaseGEEntity {
                base,
                world: self.to_gd(),
                id: entity.id(),
				world_deletion: false,
                gd_components_map: Default::default(),
            }
        });
        gd_entity.set_script(
            load::<Script>("res://addons/glecs/gd/entity.gd").to_variant(),
        );
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
    ) -> Gd<_BaseSystemBuilder>{
        let builder = _BaseSystemBuilder::new(self.to_gd());
        let mut builder_clone = builder.clone();
        let mut builder_bind = builder_clone.bind_mut();
        builder_bind.observing_events = vec![self.get_or_add_tag_entity(event)];
        builder_bind.build_type = BuildType::Observer;
        builder
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

    #[func]
    fn _new_system(&self, pipeline: Variant) -> Gd<_BaseSystemBuilder> {
        let mut builder = _BaseSystemBuilder::new(self.to_gd());
        builder.bind_mut().pipeline = pipeline;
        builder
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
                let def = self.component_definitions.insert(def);
                Callable::from_object_method(key, "_on_registered")
                    .callv(Array::default());
                def
            }
        }
    }

    pub(crate) fn new_observer_from_builder(
        &mut self,
        builder: &mut _BaseSystemBuilder,
        callable: Callable,
    ) {
        // Create contex
        let context = Box::new(ScriptSystemContext::new(
            callable.clone(),
            self,
            &builder.description.filter,
            std::mem::take(&mut builder.terms).into_boxed_slice(),
            Box::default(),
        ));

        // Create observer
        let mut observer_desc = flecs::ecs_observer_desc_t {
            events: [0;8],
            filter: builder.description.filter,
            callback: Some(Self::raw_system_iteration),
            ctx: Box::leak(context) as *mut ScriptSystemContext as *mut c_void,
            ctx_free: Some(Self::raw_system_drop),
            .. unsafe { MaybeUninit::zeroed().assume_init() }
        };

        // Set events to observe in observer
        builder.observing_events.truncate(observer_desc.events.len());
        for (i, event_id) in
            builder.observing_events.iter().enumerate()
        {
            observer_desc.events[i] = *event_id;
        }

        // Initialize observer
        let observer_id = unsafe { flecs::ecs_observer_init(
            self.world.raw(),
            &observer_desc,
        ) };
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

    pub(crate) fn new_system_from_builder(
        &mut self,
        builder: &mut _BaseSystemBuilder,
        callable: Callable,
    ) {
        let Some(pipeline_def) = self
            .get_pipeline(builder.pipeline.clone())
            else {
                show_error!(
                    "Failed to add system",
                    "Noo pipeline with identifer {} was defined.",
                    builder.pipeline,
                );
                return
            };

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
        let context = Box::new(
            ScriptSystemContext::new(
                callable.clone(),
                self,
                &builder.description.filter,
                std::mem::take(&mut builder.terms).into_boxed_slice(),
                value_getters,
            )
        );

        // Create system
        let sys_desc = flecs::ecs_system_desc_t {
            query: builder.description,
            callback: Some(Self::raw_system_iteration),
            ctx: Box::leak(context) as *mut ScriptSystemContext as *mut c_void,
            ctx_free: Some(Self::raw_system_drop),
            .. unsafe { MaybeUninit::zeroed().assume_init() }
        };

        // Initialize system
        let sys_id = unsafe { flecs::ecs_system_init(
            self.world.raw(),
            &sys_desc,
        ) };

        // Set system pipeline
        unsafe { flecs::ecs_add_id(
            self.world.raw(),
            sys_id,
            pipeline_def.flecs_id,
        ) };
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
        parameters: &Vec<ComponetProperty>,
    ) -> Layout {
        let mut size = 0;
        for (property) in parameters {
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
                    // TODO: Optimize away box allocation
					.get_data_fn_ptr = Box::new(move |_self| {
                        data
                    });
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
    /// The arguments passed to the system.
    system_args: Array<Variant>,
    terms_buffer: Box<[flecs::ecs_term_t]>,
    /// Holds the accesses stored in `sysatem_args` for quicker access.
    term_accesses: Box<[Gd<_BaseGEComponent>]>,
    world: Gd<_BaseGEWorld>,
    /// A list of getters for extra arguments in a pipeline.
    additional_arg_getters: Box<[Callable]>,
} impl ScriptSystemContext {
    fn new(
        callable: Callable,
        world: &mut _BaseGEWorld,
        filter: &flecs::ecs_filter_desc_t,
        terms_buffer: Box<[flecs::ecs_term_t]>,
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

        let raw_terms = unsafe { std::slice::from_raw_parts(
            filter.terms_buffer,
            filter.terms_buffer_count as usize,
        ) };

        // Create component accesses
        let mut tarm_accesses: Vec<Gd<_BaseGEComponent>> = vec![];
        for (term_i, term) in raw_terms.iter().enumerate() {
            // TODO: Handle different term operations
            match term.oper {
                flecs::ecs_oper_kind_t_EcsAnd => {},
                flecs::ecs_oper_kind_t_EcsOr => {
                    todo!("Handle \"or\" case")
                },
                flecs::ecs_oper_kind_t_EcsNot => { continue },
                flecs::ecs_oper_kind_t_EcsOptional => {
                    todo!("Handle \"or\" case")
                },
                _ => continue,
            }

            let term_script = world
                .component_definitions
                .get_script(&term.id)
                .unwrap();

            let mut compopnent_access = Gd::from_init_fn(|base| {
                let base_comp = _BaseGEComponent {
                    base,
                    world: world.to_gd(),
                    get_data_fn_ptr: _BaseGEComponent::new_empty_data_getter(),
                    component_definition: world
                        .get_or_add_component(&term_script),
                };
                base_comp
            });
            compopnent_access
                .bind_mut()
                .base_mut()
                .set_script(term_script.to_variant());

            compopnent_access.set_script(term_script.to_variant());
            args.push(compopnent_access.to_variant());
            tarm_accesses.push(compopnent_access);
        }
        let term_args_fast = tarm_accesses
            .into_boxed_slice();

        Self {
            callable: callable,
            system_args: args,
            term_accesses: term_args_fast,
            world: world.to_gd(),
            terms_buffer,
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