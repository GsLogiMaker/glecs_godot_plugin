
use std::alloc::Layout;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ffi::CString;
use std::hash::Hash;
use std::mem::size_of;
use std::ptr::NonNull;
use std::rc::Rc;

use flecs::bindings::*;
use flecs::EntityId;
use godot::engine::notify::NodeNotification;
use godot::engine::Engine;
use godot::engine::Script;
use godot::prelude::*;

use crate::component::_GlecsBaseComponent;
use crate::component_definitions::GdComponentData;
use crate::component_definitions::ComponentProperty;
use crate::entity::_GlecsBaseEntity;
use crate::gd_bindings::QueryIterationContext;
use crate::gd_bindings::_GlecsBindings;
use crate::gd_bindings::_GlecsComponents;
use crate::gd_bindings::_GlecsQueries;
use crate::module::_GlecsBaseModule;
use crate::prefab::PrefabDefinition;
use crate::queries::_GlecsBaseQueryBuilder;
use crate::util::script_inherets;
use crate::Int;
use crate::TYPE_SIZES;

pub(crate) fn load_world_obj_script() -> Variant {
    load::<Script>("res://addons/glecs/gd/world_object.gd")
        .to_variant()
}

pub(crate) fn load_module_script() -> Gd<Script> {
    load::<Script>("res://addons/glecs/gd/module.gd")
}

#[derive(GodotClass)]
#[class(base=Object)]
pub struct _GlecsBaseWorld {
    pub(crate) base: Base<Object>,
    pub(crate) world: NonNull<ecs_world_t>,
    prefabs: HashMap<Gd<Script>, Rc<PrefabDefinition>>,
    // / Maps Variant identifiers to entity IDs
    mapped_entities: HashMap<VariantKey, EntityId>,
    pipelines: HashMap<EntityId, Rc<RefCell<PipelineDefinition>>>,
}
#[godot_api]
impl _GlecsBaseWorld {

    /// Returns the name of the Script that was registered with the world.
    #[func(gd_self)]
    fn get_component_name(
        this: Gd<Self>,
        script: Gd<Script>,
    ) -> StringName {
        let c_id = Self::get_or_add_component_gd(this.clone(), script);
        _GlecsBindings::get_name(this, c_id).into()
    }

    #[func]
    fn _entity_from_flecs_id(&mut self, flecs_id:EntityId) -> Gd<_GlecsBaseEntity> {
        let gd_entity = Gd::from_init_fn(|base| {
            _GlecsBaseEntity {
                base,
                world: self.to_gd(),
                id: flecs_id,
            }
        });

        gd_entity
    }

    #[func(gd_self)]
    fn _new_event_listener(this: Gd<Self>, event:Variant) -> Gd<_GlecsBaseQueryBuilder>{
        let mut local_builder = Gd::from_init_fn(_GlecsBaseQueryBuilder::init);
        let event_id = Self::_id_from_variant(this.clone(), event);
        
        // Callback for when query building is finsihed
        let builder = local_builder.clone();
        let callback = Box::new(move || {
            let builder_bind = builder.bind();
            
            let context = Box::new(QueryIterationContext::new(
                builder_bind.each_callback
                    .clone()
                    .expect("Query does not have a for_each callback set"),
                this.clone(),
                &builder_bind.get_terms(),
                Box::from([]),
            ));
            
            // Create observer
            let mut observer_desc = flecs::ecs_observer_desc_t {
                events: [0;8],
                query: builder.bind().desc,
                callback: Some(_GlecsQueries::query_iteration),
                ctx: Box::leak(context) as *mut QueryIterationContext as *mut c_void,
                ctx_free: Some(_GlecsQueries::query_iteration_contex_drop),
                .. Default::default()
            };
            observer_desc.events[0] = event_id;

            // Initialize observer
            unsafe { flecs::ecs_observer_init(
                this.bind().raw(),
                &observer_desc,
            ) };
        });
        local_builder.bind_mut().build_callback = callback;

        local_builder
    }

    #[func(gd_self)]
    fn _new_system(this: Gd<Self>, pipeline:Variant) -> Gd<_GlecsBaseQueryBuilder>{
        let mut local_builder = Gd::from_init_fn(_GlecsBaseQueryBuilder::init);
        
        // TODO: Assert ID is a pipeline
        let pipeline_id = Self::_id_from_variant(this.clone(), pipeline);
        
        // Callback for when query building is finsihed
        let builder = local_builder.clone();
        let callback = Box::new(move || {
            let builder_bind = builder.bind();
            let world_bind = this.bind();
            let raw_world = world_bind.raw();

            // Assemble value getters list
            // TODO: remove use of HashMap in favor of storing metadat directly
            //      in the flecs world.
            let pipeline_def = world_bind
                .pipelines
                .get(&pipeline_id)
                .expect("ID is not a pipeline");
            let mut additional_arg_getters = Vec::new();
            for callable in pipeline_def.borrow().extra_parameters.iter_shared() {
                additional_arg_getters.push(callable);
            }
            let value_getters = additional_arg_getters.into_boxed_slice();

            // Create iteration context
            let context = Box::new(QueryIterationContext::new(
                builder_bind.each_callback
                    .clone()
                    .expect("Query does not have a for_each callback set"),
                this.clone(),
                &builder_bind.get_terms(),
                value_getters,
            ));

            drop(world_bind);

            // Define system description
            let sys_desc = flecs::ecs_system_desc_t {
                query: builder_bind.desc,
                callback: Some(_GlecsQueries::query_iteration),
                ctx: Box::leak(context) as *mut QueryIterationContext as *mut c_void,
                ctx_free: Some(_GlecsQueries::query_iteration_contex_drop),
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
        });
        local_builder.bind_mut().build_callback = callback;

        local_builder
    }

    #[func(gd_self)]
    fn _new_pipeline(
        mut this: Gd<Self>,
        identifier: GString,
        extra_parameters: Array<Callable>,
    ) -> Gd<_GlecsBaseEntity> {
        // Get or initialize pipeline
        let pipeline_id = this.bind_mut()
            .new_pipeline(identifier, extra_parameters);
        _GlecsBaseEntity::_from(Variant::from(pipeline_id), Some(this))
    }

    /// Runs all processess associated with the given pipeline.
    #[func(gd_self)]
    fn run_pipeline(this: Gd<Self>, pipeline:Variant, delta:f32) {
        let pipeline_id = Self::get_pipeline(this.clone(), pipeline);
    
        let raw_world = this.bind().raw();
        unsafe {
            flecs::ecs_set_pipeline(raw_world, pipeline_id);
            flecs::ecs_progress(raw_world, delta);
        }
    }

    #[func(gd_self)]
    fn _load_and_set_world_script(mut this: Gd<Self>) {
        this.set_script(load_world_obj_script());
    }

    #[func(gd_self)]
    fn _register_script(this: Gd<Self>, to_register: Gd<Script>, name: GString) {
        Self::register_script_and_get(this, to_register, name);
    }

    #[func(gd_self)]
    fn _print_tree(this: Gd<Self>, parent: Int, depth: Int) {
        if depth >= 10 {
            return;
        }
        let parent = parent as EntityId;
        let depth = depth as usize;

        if unsafe { flecs::ecs_id_is_pair(parent) } {
            return
        }

        let world = this.bind();
        let depth_indent = String::from("-   ").repeat(depth);
        let name = _GlecsBindings
            ::get_name_cstr_from_ref(&world, parent);
        godot_print!("{}{:?} id = {}", depth_indent, name, parent);

        if parent == unsafe { flecs::EcsAny } {
            return
        }
        if parent == unsafe { flecs::EcsWildcard } {
            return
        }

        let mut iter = unsafe { flecs::ecs_children(world.raw(), parent) };
        while unsafe { flecs::ecs_children_next(&mut iter) } {
            for i in 0..(iter.count as usize) {
                let child = unsafe { *iter.entities.add(i) };
                Self::_print_tree(
                    this.clone(),
                    child as Int,
                    (depth+1) as Int,
                );
            }
        }
    }

    #[func(gd_self)]
    fn pair(this: Gd<Self>, relation:Variant, target:Variant) -> i64 {
        let left = Self::_id_from_variant(this.clone(), relation);
        let right = Self::_id_from_variant(this, target);
        let pair = _GlecsBindings::pair(left, right);
        pair as i64
    }

    #[func]
    pub fn _get_global() -> Gd<Self> {
        Engine::singleton()
            .get_singleton("GlecsSingleton".into())
            .unwrap()
            .cast::<Self>()
    }

    #[func(gd_self)]
    /// Converts a [`Variant`] value to an EntityId in the most suitable way
    /// for the given [`Variant`] type.
    pub(crate) fn _id_from_variant(
        this: Gd<Self>,
        entity: Variant,
    ) -> EntityId {
        const VT_OBJECT:VariantType = VariantType::OBJECT;
        const VT_VECTOR2:VariantType = VariantType::VECTOR2;
        const VT_VECTOR2I:VariantType = VariantType::VECTOR2I;
        const VT_INT:VariantType = VariantType::INT;
        const VT_FLOAT:VariantType = VariantType::FLOAT;
        const VT_STRING:VariantType = VariantType::STRING;
        const VT_STRINGNAME:VariantType = VariantType::STRING_NAME;
        const VT_NIL:VariantType = VariantType::NIL;
        let this_clone = this.clone();
        let from_clone = entity.clone();
        let id = match entity.get_type() {
            VT_OBJECT => {
                if let Ok(e) =
                    entity.try_to::<Gd<_GlecsBaseEntity>>()
                {
                    return e.bind().id
                }
                if let Ok(e) = entity.try_to::<Gd<Script>>() {
                    if let Some(e) =  Self::get_tag_entity(
                        &this.bind(),
                        entity,
                    ) {
                        return e
                    }
                    return Self::register_script_and_get(
                        this,
                        e,
                        "".into(),
                    )
                }
                if let Ok(_) =
                    entity.try_to::<Gd<_GlecsBaseComponent>>()
                {
                    panic!("Too ambiguous to get an ID from a component.")
                }
                
                0
            },
            VT_VECTOR2 => {
                let veci = entity.to::<Vector2>();
                let (x, y) = (veci.x as EntityId, veci.y as EntityId);
                _GlecsBindings::pair(x, y)
            },
            VT_VECTOR2I => {
                let veci = entity.to::<Vector2i>();
                let (x, y) = (veci.x as EntityId, veci.y as EntityId);
                _GlecsBindings::pair(x, y)
            },
            VT_INT => entity.to::<i64>() as EntityId,
            VT_FLOAT => entity.to::<f64>() as EntityId,
            VT_STRING => _GlecsBindings::lookup(this, entity.to::<GString>()),
            VT_STRINGNAME => _GlecsBindings::lookup(this, entity.to::<StringName>().into()),
            VT_NIL => 0,
            _ => 0,
        };

        if !_GlecsBindings::id_is_alive(this_clone.clone(), id) {
            panic!("Value \"{from_clone}\" does not convert to a valid Entity. Converted to id: {id}")
        }

        id
    }

    pub(crate) fn get_component(
        &mut self,
        key: Gd<Script>,
    ) -> Option<EntityId> {
        todo!()
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
        drop(world_bind);
        
        // Define new component
        _GlecsComponents::define(
            this.clone(),
            key.clone(),
            "".into(),
        )
    }

    fn do_registered_callback(this: Gd<Self>, target: Gd<Script>) {
        // Call _on_registerd
        let mut args = VariantArray::new();
        args.push(this.to_variant());
        let callable = Callable
            ::from_object_method(&target, "_registered");
        callable.callv(args);
    }

    pub(crate) fn get_pipeline(
        this: Gd<Self>,
        identifier:Variant,
    ) -> EntityId {
        let entity_id = Self::_id_from_variant(
            this.clone(),
            identifier.clone(),
        );
        assert!(this.bind().is_id_pipeline(entity_id));
        entity_id
    }

    fn new_pipeline(
        &mut self,
        identifier: GString,
        extra_parameters: Array<Callable>,
    ) -> EntityId {
        // Get or initialize pipeline
        let pipeline_id = _GlecsBindings::new_id_from_ref(self);
        
        let mut system_query = flecs
            ::ecs_query_desc_t::default();
        system_query.terms[0] = flecs::ecs_term_t {
            id: pipeline_id,
            ..Default::default()
        };

        unsafe { flecs::ecs_pipeline_init(
            self.raw(),
            &flecs::ecs_pipeline_desc_t {
                entity: pipeline_id,
                query: system_query,
            },
        ) };
        assert!(unsafe {
            flecs::ecs_has_id(self.raw(), pipeline_id, flecs::FLECS_IDEcsPipelineID_)
        });
        _GlecsBindings::set_name_from_ref(self, pipeline_id, identifier);

        let def = PipelineDefinition {
            extra_parameters,
            flecs_id: pipeline_id,
        };

        let pipeline_def = Rc::new(RefCell::new(def));
        self.pipelines.insert(pipeline_id, pipeline_def.clone());

        pipeline_id
    }

    pub(crate) fn is_id_pipeline(&self, id: EntityId) -> bool {
        unsafe { flecs::ecs_has_id(
            self.raw(),
            id,
            flecs::FLECS_IDEcsPipelineID_,
        ) }
    }

    pub(crate) fn get_pipeline_definition(
        &self,
        pipeline_id: EntityId,
    ) -> &Rc<RefCell<PipelineDefinition>> {
        self.pipelines.get(&pipeline_id).expect("Could not find pipeline")
    }

    fn register_script_and_get(
        this: Gd<Self>,
        mut script: Gd<Script>,
        name: GString,
    )-> EntityId {
        let world_ptr = this.bind().raw();

        let pre_fn_scope = unsafe { ecs_get_scope(world_ptr) };

        let name_gstring = match () {
            _ if name.len() != 0 => {
                // A name was provided, no assumptions needed
                let curr_scope = unsafe { ecs_get_scope(world_ptr) };
                if curr_scope == 0 {
                    let scripts = Self::_id_from_variant(
                        this.clone(),
                        "Glecs/Scripts".to_variant(),
                    );
                    unsafe { ecs_set_scope(world_ptr, scripts)  };
                }
                name
            },

            _ if script.get_path().len() == 0 => {
                // Script has no name, and no path, use ID of script instance
                format!("Script#{}", script.instance_id().to_i64()).into()
            },

            _ => {
                // No name was provided, assume script is a .gd file
                let path = script.get_path();
    
                let scripts = Self::_id_from_variant(
                    this.clone(),
                    "Glecs/Scripts".to_variant(),
                );
                
                let path_string = path.to_string();
                let path = path_string
                    .split("res://")
                    .nth(1)
                    .unwrap()
                    .split("/");
                unsafe { ecs_set_scope(world_ptr, scripts) }; // This path should leak forward to when the entity for this script is created, then be reverted at the end of the function
                for folder in path {
                    if folder.contains(".") {
                        break;
                    }
                    let cstring = CString::new(folder).unwrap();
                    let mut found = unsafe { ecs_lookup_child(
                        world_ptr,
                        ecs_get_scope(world_ptr),
                        cstring.as_ptr(),
                    )};
                    if found == 0 {
                        found = _GlecsBindings::module_init(
                            this.clone(),
                            folder.into(),
                            0,
                        );
                        _GlecsBindings::add_pair(
                            this.clone(),
                            found,
                            unsafe { EcsChildOf },
                            unsafe { ecs_get_scope(world_ptr) },
                        );
                    }
                    // This scope is reset at the end of the funciton
                    unsafe { ecs_set_scope(world_ptr, found) };
                }
    
                let name_str =  path_string
                    .split("/")
                    .last()
                    .unwrap();
                GString::from(name_str)
            },
        };

        let pre_check_scope = unsafe { ecs_get_scope(world_ptr) };
        unsafe { ecs_set_scope(world_ptr, 0) };
        let lookup = _GlecsBindings::lookup_child(
            this.clone(),
            pre_check_scope,
            name_gstring.clone(),
        );
        if lookup != 0 {
            panic!(
                "Failed to register the script {}. An entity with the name \"{}\" has already been defined under parent \"{}\".",
                script,
                name_gstring,
                pre_check_scope,
            );
        }
        unsafe { ecs_set_scope(world_ptr, pre_check_scope) };
        
        let script_type = script.get_instance_base_type()
            .to_string();

        let item_id = match script_type {
            t if t == _GlecsBaseComponent::class_name().as_str() => {
                // Script is a component
                Self::get_or_add_component_gd(
                    this.clone(),
                    script.clone(),
                )
            },
            t if t == _GlecsBaseEntity::class_name().as_str() => {
                // Script is an entity or module
                if script_inherets(script.clone(), load_module_script()) {
                    // Script is a module
                    let has_id = Self::get_tag_entity(
                        &this.bind(),
                        script.to_variant(),
                    ).is_some();
                    let id = Self::get_or_add_tag_entity(
                        this.clone(),
                        script.to_variant(),
                    );
                    if !has_id {
                        _GlecsBindings::module_init(
                            this.clone(),
                            name_gstring.clone(),
                            id,
                        );
                        _GlecsBindings::add_pair(
                            this.clone(),
                            id,
                            unsafe { EcsChildOf },
                            unsafe { ecs_get_scope(world_ptr) },
                        );

                        // Register sub-classes and imported scripts
                        let old_scope = unsafe { ecs_get_scope(world_ptr) };
                        unsafe { ecs_set_scope(world_ptr, id) };
                        for (key, value) in
                            script.get_script_constant_map().iter_shared()
                        {
                            let Ok(sub_script) = value
                                .try_to::<Gd<Script>>()
                                else { continue };
                            Self::_register_script(
                                this.clone(),
                                sub_script,
                                key.to::<GString>(),
                            );
                        }
                        unsafe { ecs_set_scope(world_ptr, old_scope) };
                    }

                    id
                } else {
                    // Script is an entity
                    let entity = Self::get_or_add_tag_entity(
                        this.clone(),
                        script.to_variant(),
                    );
                    _GlecsBindings::add_pair(
                        this.clone(),
                        entity,
                        _GlecsBindings::_flecs_child_of(),
                        unsafe { ecs_get_scope(world_ptr) },
                    );
                    entity
                }
            },
            _ => panic!(
                "Attempted to register a non-entity script in a Glecs world. Only {}, {}, and {} scritps can be registed.",
                _GlecsBaseComponent::class_name(),
                _GlecsBaseEntity::class_name(),
                _GlecsBaseModule::class_name(),
            ),
        };

        _GlecsBindings::set_name(
            this.clone(),
            item_id, name_gstring.clone(),
        );
        
        unsafe { ecs_set_scope(world_ptr, pre_fn_scope) };
        item_id
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
            let new_id = _GlecsBindings::new_id(this.clone());
            Self::add_tag_entity(this, key, new_id);
            new_id
        })
    }

    pub(crate) fn layout_from_properties(
        parameters: &Vec<ComponentProperty>,
    ) -> Layout {
        let mut size = 0;
        for property in parameters {
            size += TYPE_SIZES[property.gd_type_id.ord() as usize];
        }
        Layout::from_size_align(size, 8).unwrap()
    }

    /// Returns a raw pointer to the Flecs world
    pub(crate) fn raw(&self) -> *mut flecs::ecs_world_t {
        self.world.as_ptr()
    }
}

#[godot_api]
impl IObject for _GlecsBaseWorld {
    fn init(base: Base<Object>) -> Self {
        let fl_world = NonNull::new(unsafe { ecs_init() })
            .unwrap();
        let mut world = Self {
            base,
            world: fl_world,
            mapped_entities: Default::default(),
            prefabs: Default::default(),
            pipelines: Default::default(),
        };

        let component_properties =_GlecsComponents::_define_raw(
            &world,
            size_of::<GdComponentData>() as i32,
            &CString::new(GdComponentData::name()).unwrap(),
        );
        // TODO: Add hooks for ComponentProperties

        // Make temporary delta time getter callables
        let process_time = Callable::invalid();
        let physics_process_time = Callable::invalid();

        let glecs_id = _GlecsBindings::set_name_from_ref(
            &world,
            0,
            "Glecs".into(),
        );

        // Add OnInit event
        let on_init = _GlecsBindings::new_id_from_ref(&world);
        _GlecsBindings::set_name_c(
            &world,
            on_init,
            CString::new("OnInit").unwrap(),
        );
        _GlecsBindings::add_pair_from_ref(
            &world,
            on_init,
            _GlecsBindings::_flecs_child_of(),
            glecs_id,
        );

        // Add custom OnSet event
        let custom_on_set = _GlecsBindings::new_id_from_ref(&world);
        _GlecsBindings::add_pair_from_ref(
            &world,
            custom_on_set,
            _GlecsBindings::_flecs_child_of(),
            glecs_id,
        );
        _GlecsBindings::set_name_c(
            &world,
            custom_on_set,
            CString::new("OnSet").unwrap(),
        );

        // Add process pipeline
        let process = world.new_pipeline(
            "process".into(),
            // Temporary callables should work in a pinch, but are replaced
            // with more appropriate ones in _GlecsBaseWorldNode
            Array::from_iter((0..1).map(|_| process_time.clone())),
        );
        _GlecsBindings::add_pair_from_ref(
            &world,
            process,
            _GlecsBindings::_flecs_child_of(),
            glecs_id,
        );

        // Add physics_process pipeline
        let physics_process = world.new_pipeline(
            "physics_process".into(),
            // Temporary callables should work in a pinch, but are replaced
            // with more appropriate ones in _GlecsBaseWorldNode
            Array::from_iter((0..1).map(|_| physics_process_time.clone())),
        );
        _GlecsBindings::add_pair_from_ref(
            &world,
            physics_process,
            _GlecsBindings::_flecs_child_of(),
            glecs_id,
        );

        // Add Scripts entity
        let scripts = _GlecsBindings::new_id_from_ref(&world);
        _GlecsBindings::set_name_from_ref(
            &world,
            scripts,
            "Scripts".into(),
        );
        _GlecsBindings::add_pair_from_ref(
            &world,
            scripts,
            _GlecsBindings::_flecs_child_of(),
            glecs_id,
        );
        
        world
    }
} impl Drop for _GlecsBaseWorld {
    fn drop(&mut self) {
        let raw = self.raw();
        unsafe { ecs_fini(raw) };
    }
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct _GlecsBaseWorldNode {
    base: Base<Node>,
    glecs_world: Gd<_GlecsBaseWorld>,
}
#[godot_api]
impl _GlecsBaseWorldNode {
        /// Returns the name of the Script that was registered with the world.
        #[func]
        fn get_component_name(
            &self,
            script: Gd<Script>,
        ) -> StringName {
            _GlecsBaseWorld::get_component_name(self.glecs_world.clone(), script)
        }
    
        #[func]
        fn _entity_from_flecs_id(
            &self,
            flecs_id: EntityId,
        ) -> Gd<_GlecsBaseEntity> {
            _GlecsBaseWorld::_entity_from_flecs_id(&mut self.glecs_world.clone().bind_mut(), flecs_id)
        }
    
        #[func]
        fn _new_event_listener(
            &self,
            event: Variant,
        ) -> Gd<_GlecsBaseQueryBuilder>{
            _GlecsBaseWorld::_new_event_listener(self.glecs_world.clone(), event)
        }
    
        #[func]
        /// Converts a [`Variant`] value to an EntityId in the most suitable way
        /// for the given [`Variant`] type.
        pub(crate) fn _id_from_variant(
            &self,
            entity: Variant,
        ) -> EntityId {
            _GlecsBaseWorld::_id_from_variant(
                self.as_object().clone(),
                entity,
            )
        }

        #[func(gd_self)]
        fn _new_pipeline(
            this: Gd<Self>,
            identifier: GString,
            extra_parameters: Array<Callable>,
        ) -> Gd<_GlecsBaseEntity> {
            _GlecsBaseWorld::_new_pipeline(
                this.bind().as_object().clone(),
                identifier,
                extra_parameters,
            )
        }

        #[func]
        fn _register_script(&self, to_register: Gd<Script>, name: GString) {
            _GlecsBaseWorld::_register_script(
                self.glecs_world.clone(),
                to_register,
                name,
            );
        }
    
        /// Runs all processess associated with the given pipeline.
        #[func]
        fn run_pipeline(
            &self,
            pipeline:Variant,
            delta:f32,
        ) {
            _GlecsBaseWorld::run_pipeline(self.as_object(), pipeline, delta)
        }
    
        #[func]
        fn _new_system(&self, pipeline: Variant) -> Gd<_GlecsBaseQueryBuilder> {
            _GlecsBaseWorld::_new_system(self.glecs_world.clone(), pipeline)
        }
    
        #[func]
        fn pair(&self, relation:Variant, target:Variant) -> i64 {
            _GlecsBaseWorld::pair(self.glecs_world.clone(), relation, target)

        }
    
        #[func]
        /// Converts a [`Variant`] value to an EntityId in the most suitable way
        /// for the given [`Variant`] type.
        pub(crate) fn variant_to_entity_id(
            &self,
            from:Variant,
        ) -> EntityId {
            _GlecsBaseWorld::_id_from_variant(self.glecs_world.clone(), from)
        }

        #[func]
        fn as_object(&self) -> Gd<_GlecsBaseWorld> {
            self.glecs_world.clone()
        }
}
#[godot_api]
impl INode for _GlecsBaseWorldNode {
    fn init(base: Base<Node>) -> Self {
        let mut glecs_world = Gd::<_GlecsBaseWorld>
            ::from_init_fn(_GlecsBaseWorld::init);
        
        glecs_world.set_script(load_world_obj_script());
        
        Self {
            base,
            glecs_world,
        }
    }

    fn ready(&mut self) {
        let process_time = Callable::from_object_method(
            &self.to_gd(),
            StringName::from("get_process_delta_time"),
        );
        let physics_process_time = Callable::from_object_method(
            &self.to_gd(),
            StringName::from("get_physics_process_delta_time"),
        );

        let process = _GlecsBaseWorld::_id_from_variant(
            self.glecs_world.clone(),
            "Glecs/process".to_variant(),
        );
        let physics_process = _GlecsBaseWorld::_id_from_variant(
            self.glecs_world.clone(),
            "Glecs/physics_process".to_variant(),
        );

        // Add delta to process
        self.glecs_world.bind_mut()
            .get_pipeline_definition(process)
            .borrow_mut()
            .extra_parameters
            .set(0, process_time);

        // Add delta to physics_process
        self.glecs_world.bind_mut()
            .get_pipeline_definition(physics_process)
            .borrow_mut()
            .extra_parameters
            .set(0, physics_process_time);
    }

    fn on_notification(&mut self, what: NodeNotification) {
        match what {
            NodeNotification::PREDELETE => {
                Gd::free(self.glecs_world.clone());
            },
            _ => {},
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