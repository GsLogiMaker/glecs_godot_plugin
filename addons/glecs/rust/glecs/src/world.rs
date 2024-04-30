
use std::alloc::Layout;
use std::collections::HashMap;
use std::ffi::c_void;
use std::hash::Hash;
use std::mem::MaybeUninit;
use std::rc::Rc;

use flecs::EntityId;
use flecs::Iter;
use flecs::World as FlWorld;
use godot::engine::notify::NodeNotification;
use godot::engine::Script;
use godot::prelude::*;

use crate::component::_GlecsComponent;
use crate::component_definitions::ComponetDefinition;
use crate::component_definitions::ComponetProperty;
use crate::entity::EntityLike;
use crate::entity::_GlecsEntity;
use crate::event::_GlecsEvent;
use crate::prefab::PrefabDefinition;
use crate::prefab::_GlecsPrefab;
use crate::prefab::PREFAB_COMPONENTS;
use crate::queries::BuildType;
use crate::queries::_GlecsSystemBuilder;
use crate::show_error;
use crate::TYPE_SIZES;

#[derive(GodotClass)]
#[class(base=Object)]
pub struct _GlecsWorld {
    pub(crate) base: Base<Object>,
    pub(crate) world: FlWorld,
    prefabs: HashMap<Gd<Script>, Rc<PrefabDefinition>>,
    /// Maps Variant identifiers to entity IDs
    mapped_entities: HashMap<VariantKey, EntityId>,
    pipelines: HashMap<EntityId, Rc<PipelineDefinition>>,
    components: HashMap<EntityId, Rc<ComponetDefinition>>,
}
#[godot_api]
impl _GlecsWorld {
    /// Returns the name of the Script that was registered with the world.
    #[func]
    fn get_component_name(
        &mut self,
        script: Gd<Script>,
    ) -> StringName {
        let c_id = self.get_or_add_component(script);
        let c_desc = self.get_component_description(c_id)
            .unwrap();

        c_desc.name.clone()
    }

    #[func]
    fn _entity_from_flecs_id(&mut self, flecs_id:EntityId) -> Gd<_GlecsEntity> {
        let gd_entity = Gd::from_init_fn(|base| {
            _GlecsEntity {
                base,
                world: self.to_gd(),
                id: flecs_id,
            }
        });

        gd_entity
    }


    /// Creates a new entity in the world.
    #[func(gd_self)]
    fn _new_entity(
        mut this: Gd<Self>,
        name: String,
        with_components:Array<Variant>,
    ) -> Gd<_GlecsEntity> {
        let entity = this.bind_mut().world.entity();

        for component in with_components.iter_shared() {
            let component_id = _GlecsWorld::_id_from_variant(
                this.clone(),
                component,
            );
            _GlecsEntity::add_component_raw(
                this.clone(),
                entity.id(),
                component_id,
                Variant::nil(),
            );
        }

        // Create Godot wrapper
        let this_clone = this.clone();
        let mut gd_entity = Gd::from_init_fn(|base| {
            _GlecsEntity {
                base,
                world: this_clone,
                id: entity.id(),
            }
        });
        
        gd_entity.set_name(name);
        gd_entity.set_script(
            load::<Script>("res://addons/glecs/gd/entity.gd").to_variant(),
        );
        
        gd_entity
    }

    /// Creates a new entity in the world. 
    #[func(gd_self)]
    fn new_entity_with_prefab(
        mut this: Gd<Self>,
        name: String,
        prefab: Gd<Script>,
    ) -> Gd<_GlecsEntity> {
        let gd_entity = Self
            ::_new_entity(this.clone(), name, Array::default());
        let e_id = gd_entity.bind().get_flecs_id();
        
        // let prefab_id = this.bind_mut().entity_from_variant(prefab);
        let prefab_id = Self::_id_from_variant(
            this.clone(),
            prefab.to_variant(),
        );

        let this_bind = this.bind_mut();
        let raw_world = this_bind.world.raw();
        drop(this_bind);

        unsafe { flecs::ecs_add_id(
            raw_world,
            e_id,
            flecs::ecs_pair(flecs::EcsIsA, prefab_id),
        ) };

        gd_entity
    }


    #[func(gd_self)]
    fn _new_event_listener(
        this: Gd<Self>,
        event: Variant,
    ) -> Gd<_GlecsSystemBuilder>{
        let event = Self::_id_from_variant(this.clone(), event);
        let builder = _GlecsSystemBuilder::new(this);
        let mut builder_clone = builder.clone();
        let mut builder_bind = builder_clone.bind_mut();
        builder_bind.observing_events = vec![event];
        builder_bind.build_type = BuildType::Observer;
        builder
    }

    #[func(gd_self)]
    fn _new_pipeline(
        mut this: Gd<Self>,
        identifier: Variant,
        extra_parameters: Array<Callable>,
    ) {
        // Get or initialize pipeline
        let pipeline_id = Self::get_or_add_tag_entity(
            this.clone(),
            identifier.clone(),
        );
        if this.bind().pipelines.get(&pipeline_id).is_some() {
            show_error!(
                "Failed to create pipeline",
                "Pipeline identified by {:?} has already been defined",
                identifier
            );
        }
        
        let mut system_query = flecs
            ::ecs_query_desc_t::default();
        system_query.filter.terms[0] = flecs::ecs_term_t {
            id: pipeline_id,
            ..Default::default()
        };

        unsafe { flecs::ecs_pipeline_init(
            this.bind().raw(),
            &flecs::ecs_pipeline_desc_t {
                entity: pipeline_id,
                query: system_query,
            },
        ) };

        let def = PipelineDefinition {
            extra_parameters,
            flecs_id: pipeline_id,
        };

        let pipeline_def = Rc::new(def);
        this.bind_mut().pipelines.insert(pipeline_id, pipeline_def.clone());
    }

    /// Runs all processess associated with the given pipeline.
    #[func]
    fn run_pipeline(&self, pipeline:Variant, delta:f32) {
        let pipeline_id = self.get_pipeline(pipeline);
    
        let raw_world = self.world.raw();
        unsafe {
            flecs::ecs_set_pipeline(raw_world, pipeline_id);
            flecs::ecs_progress(raw_world, delta);
        }
    }

    #[func(gd_self)]
    fn _new_system(this: Gd<Self>, pipeline: Variant) -> Gd<_GlecsSystemBuilder> {
        let mut builder = _GlecsSystemBuilder::new(this);
        builder.bind_mut().pipeline = pipeline;
        builder
    }

    #[func(gd_self)]
    fn pair(this: Gd<Self>, relation:Variant, target:Variant) -> i64 {
        let left = Self::_id_from_variant(this.clone(), relation);
        let right = Self::_id_from_variant(this, target);
        let pair = flecs::ecs_pair(left, right);
        pair as i64
    }

    #[func(gd_self)]
    /// Converts a [`Variant`] value to an EntityId in the most suitable way
    /// for the given [`Variant`] type.
    pub(crate) fn _id_from_variant(
        this: Gd<Self>,
        entity: Variant,
    ) -> EntityId {
        use VariantType::Object as VTObject;
        use VariantType::Vector2 as VTVector2;
        use VariantType::Vector2i as VTVector2i;
        use VariantType::Int as VTInt;
        use VariantType::Float as VTFloat;
        use VariantType::String as VTString;
        use VariantType::StringName as VTStringName;
        use VariantType::Nil as VTNil;
        let this_clone = this.clone();
        let from_clone = entity.clone();
        let id = match entity.get_type() {
            VTObject => {
                if let Ok(e) =
                    entity.try_to::<Gd<_GlecsEntity>>()
                {
                    return e.bind().id
                }
                if let Ok(e) =
                    entity.try_to::<Gd<_GlecsComponent>>()
                {
                    return e.bind().component_definition.flecs_id
                }
                if let Ok(e) =
                    entity.try_to::<Gd<_GlecsEvent>>()
                {
                    return e.bind()._id
                }
                if let Ok(e) =
                    entity.try_to::<Gd<_GlecsPrefab>>()
                {
                    return e.bind().flecs_id
                }
                if let Ok(e) = entity.try_to::<Gd<Script>>() {
                    let component_type = _GlecsComponent::class_name()
                        .to_string_name();
                    let entity_type = _GlecsEntity::class_name()
                        .to_string_name();
                    match e.get_instance_base_type() {
                        type_name if type_name == component_type => {
                            // Component script
                            return Self::get_or_add_component_gd(this, e)
                        },
                        type_name if type_name == entity_type => {
                            // Entity script
                            return Self::get_or_add_tag_entity(
                                this,
                                e.to_variant(),
                            )
                        },
                        _ => {},
                    }
                    if e.get_instance_base_type() == component_type {
                    }
                }
                
                panic!("Object is not an Entity")
            },
            VTVector2 => {
                let veci = entity.to::<Vector2>();
                let (x, y) = (veci.x as EntityId, veci.y as EntityId);
                flecs::ecs_pair(x, y)
            },
            VTVector2i => {
                let veci = entity.to::<Vector2i>();
                let (x, y) = (veci.x as EntityId, veci.y as EntityId);
                flecs::ecs_pair(x, y)
            },
            VTInt => entity.to::<i64>() as EntityId,
            VTFloat => entity.to::<f64>() as EntityId,
            VTString => Self::get_or_add_tag_entity(this, entity),
            VTStringName => Self::get_or_add_tag_entity(this, entity),
            VTNil => panic!("Null is not an Entity"),
            _ => panic!("Variant is not an Entity"),
        };

        if !unsafe { flecs::ecs_id_is_valid(this_clone.bind().raw(), id) } {
            panic!("Value \"{from_clone}\" does not convert to a valid Entity.")
        }

        id
    }

    pub(crate) fn get_component(
        &mut self,
        key: Gd<Script>,
    ) -> Option<EntityId> {
        let Some(id) = self.get_tag_entity(key.to_variant())
            else { return None };
        let is_component = unsafe{ flecs::ecs_has_id(
            self.world.raw(),
            id,
            flecs::FLECS_IDEcsComponentID_,
        ) };
        if !is_component {
            return None;
        }
        Some(id)
    }

    pub(crate) fn get_or_add_component(
        &mut self,
        key: Gd<Script>,
    ) -> EntityId {
        Self::get_or_add_component_gd(self.to_gd(), key)
    }

    pub(crate) fn get_or_add_component_gd(
        mut this: Gd<Self>,
        key: Gd<Script>,
    ) -> EntityId {
        let mut world_bind = this.bind_mut();
        
        if let Some(id) = world_bind.get_component(key.clone()) {
            // Component with script already exists
            return id
        }
        
        // Create component and definition
        // (ComponetDefinition::new() creates the component)
        let def = Rc::new(ComponetDefinition::new(
            key.clone(),
            &mut world_bind,
        ));
        world_bind.components.insert(def.flecs_id, def.clone());
        drop(world_bind);
        
        Self::add_tag_entity(this, key.to_variant(), def.flecs_id);

        def.flecs_id
    }

    pub(crate) fn get_component_description(
        &self,
        key: EntityId,
    ) -> Option<Rc<ComponetDefinition>> {
        self.components.get(&key).map(|x| x.clone())
    }

    fn do_registered_callback(this: Gd<Self>, target: Gd<Script>) {
        // Call _on_registerd
        let mut args = VariantArray::new();
        args.push(this.to_variant());
        let callable = Callable
            ::from_object_method(&target, "_registered");
        callable.callv(args);
    }

    pub(crate) fn new_observer_from_builder(
        this: Gd<Self>,
        builder: &mut _GlecsSystemBuilder,
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
        unsafe { flecs::ecs_observer_init(
            this.bind().world.raw(),
            &observer_desc,
        ) };
    }
    
    pub(crate) fn get_pipeline(
        &self,
        identifier:Variant,
    ) -> EntityId {
        let entity_id = self.get_tag_entity(identifier.clone())
            .expect(&format!(
                "{} is not an entity",
                identifier,
            ));
        assert!(self.is_id_pipeline(entity_id));
        entity_id
    }

    pub(crate) fn is_id_pipeline(&self, id: EntityId) -> bool {
        unsafe { flecs::ecs_has_id(
            self.world.raw(),
            id,
            flecs::FLECS_IDEcsPipelineID_,
        ) }
    }

    pub(crate) fn get_pipeline_definition(
        &self,
        pipeline_id: EntityId,
    ) -> &Rc<PipelineDefinition> {
        self.pipelines.get(&pipeline_id).expect("Could not find pipeline")
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
        builder: &mut _GlecsSystemBuilder,
        callable: Callable,
    ) {
        let this_bound = this.bind();
        let raw_world = this_bound.world.raw();

        let pipeline_id = this_bound
            .get_pipeline(builder.pipeline.clone());
        let pipeline_def = this_bound
            .get_pipeline_definition(pipeline_id);
        
        // Create value getters list
        let mut additional_arg_getters = Vec::new();
        for callable in pipeline_def.extra_parameters.iter_shared() {
            additional_arg_getters.push(callable);
        }
        let value_getters = additional_arg_getters.into_boxed_slice();
        
        drop(this_bound);

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
            pipeline_id,
        ) };
    }

    pub(crate) fn add_tag_entity(
        mut this: Gd<Self>,
        key: Variant,
        id: EntityId,
    ) -> EntityId {
        let varint_key = VariantKey::new(key.clone());
        this.bind_mut().mapped_entities.insert(varint_key, id);

        if let Ok(s) = key.try_to::<Gd<Script>>() {
            Self::do_registered_callback(this, s);
        }

        id
    }

    pub(crate) fn get_tag_entity(&self, key:Variant) -> Option<EntityId> {
        self.mapped_entities.get(&VariantKey::new(key)).map(|x| *x)
    }

    pub(crate) fn get_or_add_tag_entity(
        mut this: Gd<Self>,
        key:Variant,
    ) -> EntityId {
        let variant_key = VariantKey::new(key.clone());
        let id_opt = this.bind_mut()
            .mapped_entities
            .get(&variant_key)
            .map(|x| *x);
        
        id_opt.unwrap_or_else(|| {
            let new_id = this.bind_mut().world.entity().id();
            Self::add_tag_entity(this, key, new_id);
            new_id
        })
    }

    pub(crate) fn layout_from_properties(
        parameters: &Vec<ComponetProperty>,
    ) -> Layout {
        let mut size = 0;
        for property in parameters {
            size += TYPE_SIZES[property.gd_type_id as usize];
        }
        Layout::from_size_align(size, 8).unwrap()
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
            let Ok(component_script) = component
                .try_to::<Gd<Script>>()
                else { continue };
                
            prefab_entt.add_id(Self::get_or_add_component_gd(
                this.clone(),
                component_script,
            ));
        }

        Rc::new(PrefabDefinition {
            script: script,
            flecs_id: prefab_entt.id(),
        })
    }

    /// Returns a raw pointer to the Flecs world
    pub(crate) fn raw(&self) -> *mut flecs::ecs_world_t {
        self.world.raw()
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

        let entity_count = unsafe {*iter_ptr}.count as usize;
		for entity_i in 0..entity_count {
            let entity = unsafe { *(*iter_ptr).entities.add(entity_i) };
            let field_count = unsafe {*iter_ptr}.field_count as usize;
            
			// Update cached component arguments
			for field_i in 0..field_count {
                let mut term_bind = context
                    .term_accesses[field_i]
                    .bind_mut();
				term_bind.entity_id = entity;
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
impl IObject for _GlecsWorld {
    fn init(base: Base<Object>) -> Self {
        let world = FlWorld::new();
        let mut gd_world = Self {
            base,
            world: world,
            components: Default::default(),
            prefabs: Default::default(),
            mapped_entities: Default::default(),
            pipelines: Default::default(),
        };

        gd_world.mapped_entities.insert(
            StringName::from("flecs.on_add").to_variant().into(),
            unsafe { flecs::EcsOnAdd },
        );
        gd_world.mapped_entities.insert(
            StringName::from("flecs.on_remove").to_variant().into(),
            unsafe { flecs::EcsOnRemove },
        );
        gd_world.mapped_entities.insert(
            StringName::from("flecs.on_set").to_variant().into(),
            unsafe { flecs::EcsOnSet },
        );
        gd_world.mapped_entities.insert(
            StringName::from("flecs.on_unset").to_variant().into(),
            unsafe { flecs::EcsUnSet },
        );
        gd_world.mapped_entities.insert(
            StringName::from("flecs.on_monitor").to_variant().into(),
            unsafe { flecs::EcsMonitor },
        );
        gd_world.mapped_entities.insert(
            StringName::from("flecs.on_delete").to_variant().into(),
            unsafe { flecs::EcsOnDelete },
        );
        gd_world.mapped_entities.insert(
            StringName::from("flecs.on_table_create").to_variant().into(),
            unsafe { flecs::EcsOnTableCreate },
        );
        gd_world.mapped_entities.insert(
            StringName::from("flecs.on_table_delete").to_variant().into(),
            unsafe { flecs::EcsOnTableDelete },
        );
        gd_world.mapped_entities.insert(
            StringName::from("flecs.on_table_empty").to_variant().into(),
            unsafe { flecs::EcsOnTableEmpty },
        );
        gd_world.mapped_entities.insert(
            StringName::from("flecs.on_table_fill").to_variant().into(),
            unsafe { flecs::EcsOnTableFill },
        );
        gd_world.mapped_entities.insert(
            StringName::from("flecs.prefab").to_variant().into(),
            unsafe { flecs::EcsPrefab },
        );
        gd_world.mapped_entities.insert(
            StringName::from("flecs.is_a").to_variant().into(),
            unsafe { flecs::EcsIsA },
        );

        gd_world
    }
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct _GlecsWorldNode {
    base: Base<Node>,
    glecs_world: Gd<_GlecsWorld>,
}
#[godot_api]
impl _GlecsWorldNode {
        /// Returns the name of the Script that was registered with the world.
        #[func]
        fn get_component_name(
            &self,
            script: Gd<Script>,
        ) -> StringName {
            _GlecsWorld::get_component_name(&mut self.glecs_world.clone().bind_mut(), script)
        }
    
        #[func]
        fn _entity_from_flecs_id(
            &self,
            flecs_id: EntityId,
        ) -> Gd<_GlecsEntity> {
            _GlecsWorld::_entity_from_flecs_id(&mut self.glecs_world.clone().bind_mut(), flecs_id)
        }
    
        /// Creates a new entity in the world.
        #[func]
        fn _new_entity(
            &self,
            name: String,
            with_components:Array<Variant>,
        ) -> Gd<_GlecsEntity> {
            _GlecsWorld::_new_entity(self.glecs_world.clone(), name, with_components)
        }
    
        /// Creates a new entity in the world. 
        #[func]
        fn new_entity_with_prefab(
            &self,
            name: String,
            prefab: Gd<Script>,
        ) -> Gd<_GlecsEntity> {
            _GlecsWorld::new_entity_with_prefab(self.glecs_world.clone(), name, prefab)

        }
    
        #[func]
        fn _new_event_listener(
            &self,
            event: Variant,
        ) -> Gd<_GlecsSystemBuilder>{
            _GlecsWorld::_new_event_listener(self.glecs_world.clone(), event)
        }
    
        #[func]
        /// Converts a [`Variant`] value to an EntityId in the most suitable way
        /// for the given [`Variant`] type.
        pub(crate) fn _id_from_variant(
            &self,
            entity: Variant,
        ) -> EntityId {
            _GlecsWorld::_id_from_variant(
                self.as_object().clone(),
                entity,
            )
        }

        #[func(gd_self)]
        fn _new_pipeline(
            this: Gd<Self>,
            identifier:Variant,
            extra_parameters: Array<Callable>,
        ) {
            _GlecsWorld::_new_pipeline(
                this.bind().as_object().clone(),
                identifier,
                extra_parameters,
            )
        }
    
        /// Runs all processess associated with the given pipeline.
        #[func]
        fn run_pipeline(
            &self,
            pipeline:Variant,
            delta:f32,
        ) {
            _GlecsWorld::run_pipeline(&mut self.glecs_world.clone().bind(), pipeline, delta)
        }
    
        #[func]
        fn _new_system(&self, pipeline: Variant) -> Gd<_GlecsSystemBuilder> {
            _GlecsWorld::_new_system(self.glecs_world.clone(), pipeline)
        }
    
        #[func]
        fn pair(&self, relation:Variant, target:Variant) -> i64 {
            _GlecsWorld::pair(self.glecs_world.clone(), relation, target)

        }
    
        #[func]
        /// Converts a [`Variant`] value to an EntityId in the most suitable way
        /// for the given [`Variant`] type.
        pub(crate) fn variant_to_entity_id(
            &self,
            from:Variant,
        ) -> EntityId {
            _GlecsWorld::_id_from_variant(self.glecs_world.clone(), from)
        }

        #[func]
        fn as_object(&self) -> Gd<_GlecsWorld> {
            self.glecs_world.clone()
        }
}
#[godot_api]
impl INode for _GlecsWorldNode {
    fn init(base: Base<Node>) -> Self {
        let mut glecs_world = Gd::<_GlecsWorld>
            ::from_init_fn(_GlecsWorld::init);
        
        glecs_world.set_script(
            load::<Script>("res://addons/glecs/gd/world_obj.gd").to_variant()
        );
        
        Self {
            base,
            glecs_world,
        }
    }

    fn on_notification(&mut self, what: NodeNotification) {
        match what {
            NodeNotification::Predelete => {
                Gd::free(self.glecs_world.clone());
            },
            _ => {},
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ScriptSystemContext {
    callable: Callable,
    /// The arguments passed to the system.
    system_args: Array<Variant>,
    /// Holds the accesses stored in `sysatem_args` for quicker access.
    term_accesses: Box<[Gd<_GlecsComponent>]>,
    /// A list of getters for extra arguments in a pipeline.
    additional_arg_getters: Box<[Callable]>,
} impl ScriptSystemContext {
    fn new(
        callable: Callable,
        world: Gd<_GlecsWorld>,
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
        let mut tarm_accesses: Vec<Gd<_GlecsComponent>> = vec![];
        for term in raw_terms.iter() {
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

            let term_description = world
                .bind()
                .get_component_description(term.id)
                .unwrap();
            let term_script = Gd::<Script>
                ::from_instance_id(term_description.script_id);

            let mut compopnent_access = Gd::from_init_fn(|base| {
                let base_comp = _GlecsComponent {
                    base,
                    entity_id: 0, // ID should be changed by the system
                    world: world.clone(),
                    component_definition: term_description.clone(),
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
pub(crate) struct VariantKey {
    variant: Variant
} impl VariantKey {
    fn new(v: Variant) -> Self {
        assert_eq!(v.is_nil(), false);
        Self { variant:v }
    }
} impl Eq for VariantKey {
} impl Hash for VariantKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Variant::hash(&self.variant).hash(state);
    }
} impl From<Variant> for VariantKey {
    fn from(value: Variant) -> Self {
        VariantKey::new(value)
    }
}