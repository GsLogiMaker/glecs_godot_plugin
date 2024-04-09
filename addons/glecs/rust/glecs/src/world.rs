
use std::alloc::Layout;
use std::collections::HashMap;
use std::ffi::c_void;
use std::hash::Hash;
use std::mem::size_of;
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::rc::Rc;

use flecs::EntityId;
use flecs::Iter;
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
use crate::queries::BuildType;
use crate::queries::_BaseSystemBuilder;
use crate::show_error;
use crate::TYPE_SIZES;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct _BaseGEWorld {
    pub(crate) node: Base<Node>,
    pub(crate) world: FlWorld,
    component_definitions: ComponentDefinitions,
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
    #[func(gd_self)]
    fn _new_entity(
        mut this: Gd<Self>,
        name: String,
        with_components:Array<Gd<Script>>,
    ) -> Gd<_BaseGEEntity> {
        let entity = this.bind_mut().world.entity();

        for component in with_components.iter_shared() {
            _BaseGEEntity::add_component_raw(
                this.clone(),
                entity.id(),
                component,
                Variant::nil(),
            );
        }

        // Create Godot wrapper
        let this_clone = this.clone();
        let mut gd_entity = Gd::from_init_fn(|base| {
            _BaseGEEntity {
                base,
                world: this_clone,
                id: entity.id(),
				world_deletion: false,
                gd_components_map: Default::default(),
            }
        });
        let mut bind = this.bind_mut();
        gd_entity.set_script(
            load::<Script>("res://addons/glecs/gd/entity.gd").to_variant(),
        );
        gd_entity.set_name_by_ref(name, &bind);
        bind.gd_entity_map.insert(entity.id(), gd_entity.clone());
        
        gd_entity
    }

    /// Creates a new entity in the world. 
    #[func(gd_self)]
    fn new_entity_with_prefab(
        mut this: Gd<Self>,
        name:String,
        prefab:Gd<Script>,
    ) -> Gd<_BaseGEEntity> {
        let gd_entity = Self
            ::_new_entity(this.clone(), name, Array::default());
        let e_id = gd_entity.bind().get_flecs_id();

        let prefab_def = Self
            ::get_or_add_prefab_definition(this.clone(), prefab);
        let this_bind = this.bind_mut();
        let raw_world = this_bind.world.raw();
        drop(this_bind);

        unsafe { flecs::ecs_add_id(
            raw_world,
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
        let event = self.get_or_add_tag_entity(event);
        let world_gd = self.to_gd();
        let builder = _BaseSystemBuilder::new(world_gd);
        let mut builder_clone = builder.clone();
        let mut builder_bind = builder_clone.bind_mut();
        builder_bind.observing_events = vec![event];
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

    #[func(gd_self)]
    fn _new_system(this: Gd<Self>, pipeline: Variant) -> Gd<_BaseSystemBuilder> {
        let mut builder = _BaseSystemBuilder::new(this);
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
        Self::get_or_add_component_gd(self.to_gd(), key)
    }

    pub(crate) fn get_or_add_component_gd(
        mut this: Gd<Self>,
        key: &Gd<Script>,
    ) -> Rc<ComponetDefinition> {
        let mut world_bind = this.bind_mut();
        let value = ComponentDefinitionsMapKey
            ::from(key)
            .get_value(&world_bind.component_definitions);
        let def = match value {
            Some(value) => value,
            None => {
                let def = ComponetDefinition::new(
                    key.clone(),
                    &mut world_bind,
                );
                let def = world_bind
                    .component_definitions
                    .insert(def);

                drop(world_bind);

                this.bind_mut();
                
                let mut args = VariantArray::new();
                args.push(this.to_variant());
                let callable = Callable
                    ::from_object_method(key, "_on_registered");

                callable.callv(args);

                def
            }
        };
        

        def
    }

    pub(crate) fn new_observer_from_builder(
        this: Gd<Self>,
        builder: &mut _BaseSystemBuilder,
        callable: Callable,
    ) {
        // Create contex
        let context = Box::new(ScriptSystemContext::new(
            callable.clone(),
            this.clone(),
            &builder.description.filter,
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
            this.bind().world.raw(),
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

    fn get_or_add_prefab_definition(mut this: Gd<Self>, script:Gd<Script>) -> Rc<PrefabDefinition> {
        if let Some(prefab_def) = this.bind().prefabs.get(&script) {
            return prefab_def.clone()
        }
        let prefab = Self
            ::new_prefab_def(this.clone(), script.clone());
        this.bind_mut().prefabs.insert(script.clone(), prefab);
        this.bind().prefabs.get(&script).unwrap().clone()
    }

    pub(crate) fn new_system_from_builder(
        this: Gd<Self>,
        builder: &mut _BaseSystemBuilder,
        callable: Callable,
    ) {
        let this_bound = this.bind();
        let raw_world = this_bound.world.raw();

        let Some(pipeline_def) = this_bound
            .get_pipeline(builder.pipeline.clone())
            else {
                show_error!(
                    "Failed to add system",
                    "Noo pipeline with identifer {} was defined.",
                    builder.pipeline,
                );
                return
            };
        
        drop(this_bound);

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
                this,
                &builder.description.filter,
                value_getters,
            )
        );

        // Create system
        let sys_desc = flecs::ecs_system_desc_t {
            query: builder.description,
            callback: Some(Self::raw_system_iteration),
            ctx: Box::leak(context) as *mut ScriptSystemContext as *mut c_void,
            ctx_free: Some(Self::raw_system_drop),
            .. Default::default()
        };

        // Initialize system
        let sys_id = unsafe { flecs::ecs_system_init(
            raw_world,
            &sys_desc,
        ) };

        // Set system pipeline
        unsafe { flecs::ecs_add_id(
            raw_world,
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
        this: Gd<Self>,
        mut script:Gd<Script>,
    ) -> Rc<PrefabDefinition> {
        let prefab_entt = this.bind().world
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
                Self::get_or_add_component_gd(this.clone(), &component).flecs_id
            );
        }

        Rc::new(PrefabDefinition {
            script: script,
            flecs_id: prefab_entt.id(),
        })
    }

    extern "C" fn raw_system_iteration(iter_ptr:*mut flecs::ecs_iter_t) {
		let context = unsafe {
            // Here we decouple the mutable reference of the context
            // from the rest of Iter.
			(
                Iter::new(iter_ptr)
                    .get_context_mut::<ScriptSystemContext>()
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

        let entity_count = unsafe {*iter_ptr}.count;
		for _entity_index in 0..entity_count {
			// Create components arguments
            let field_count = unsafe {*iter_ptr}.field_count;
			for field_i in 0i32..(field_count) {
                let mut term_bind = context
                    .term_accesses[field_i as usize]
                    .bind_mut();
                let component_size = term_bind
                    .component_definition
                    .layout
                    .size();
                let data = unsafe { NonNull::new_unchecked(
                    std::slice::from_raw_parts_mut(
                        flecs::ecs_field_w_size(iter_ptr, component_size, field_i+1) as *mut u8,
                        component_size,
                    )
                ) };

                // TODO: Optimize away box allocation
				term_bind.get_data_fn_ptr = Box::new(move |_self| {
                    data
                });
			}
			
			let _result = context.callable.callv(
				context.system_args.clone()
			);
		}
	}

    extern "C" fn raw_system_drop(context_ptr:*mut c_void) {
        let boxed = unsafe { Box::from_raw(
            context_ptr.cast::<ScriptSystemContext>()
        ) };
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
    /// Holds the accesses stored in `sysatem_args` for quicker access.
    term_accesses: Box<[Gd<_BaseGEComponent>]>,
    /// A list of getters for extra arguments in a pipeline.
    additional_arg_getters: Box<[Callable]>,
} impl ScriptSystemContext {
    fn new(
        callable: Callable,
        world: Gd<_BaseGEWorld>,
        filter: &flecs::ecs_filter_desc_t,
        additional_arg_getters: Box<[Callable]>,
    ) -> Self {
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
                    todo!("Handle \"optional\" case")
                },
                _ => continue,
            }

            let term_script = world
                .bind()
                .component_definitions
                .get_script(&term.id)
                .unwrap();

            let mut compopnent_access = Gd::from_init_fn(|base| {
                let base_comp = _BaseGEComponent {
                    base,
                    world: world.clone(),
                    get_data_fn_ptr: _BaseGEComponent::new_empty_data_getter(),
                    component_definition: _BaseGEWorld::get_or_add_component_gd(
                        world.clone(),
                        &term_script,
                    ),
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