
use flecs::*;
use flecs::EntityId;
use godot::engine::Script;
use godot::prelude::*;

use crate::world::_GlecsBaseWorld;

pub(crate) fn load_system_builder_script() -> Variant {
    load::<Script>("res://addons/glecs/gd/system_builder.gd")
        .to_variant()
}

#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct _GlecsQuery {
	pub(crate) base: Base<RefCounted>,
    pub(crate) desc: ecs_query_desc_t,
    pub(crate) term_count: usize,
}
#[godot_api]
impl _GlecsQuery {
    fn new_query() -> () {
    }
}
#[godot_api]
impl IRefCounted for _GlecsQuery {
    fn init(base:Base<RefCounted>) -> Self {
        Self {
            base,
            desc: Default::default(),
            term_count: 0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) enum BuildType {
    #[default]
    System,
    Observer,
}

#[derive(GodotClass)]
#[class(base=RefCounted, no_init)]
pub struct _GlecsBaseSystemBuilder {
    pub(crate) base: Base<RefCounted>,
    pub(crate) world: Gd<_GlecsBaseWorld>,
    pub(crate) pipeline: Variant,
    pub(crate) description: flecs::ecs_query_desc_t,
    /// Describes the kind object this builder is building.
    pub(crate) build_type: BuildType,
    /// An array of the events the final observer will trigger on. Only used
    /// when building an observer.
    pub(crate) observing_events: Vec<EntityId>,
    pub(crate) term_count: usize,
}
#[godot_api]
impl _GlecsBaseSystemBuilder {
    pub(crate) fn new(world:Gd<_GlecsBaseWorld>) -> Gd<Self> {
        let mut gd = Gd::from_init_fn(|base| {
            let builder = _GlecsBaseSystemBuilder {
                base,
                pipeline: Variant::nil(),
                world,
                description: Default::default(),
                build_type: Default::default(),
                observing_events: Default::default(),
                term_count: Default::default(),
            };
            builder
        });
        gd.set_script(load_system_builder_script());
        gd
    }

    #[constant]
    pub const INOUT_MODE_DEFAULT:flecs::ecs_inout_kind_t = flecs::ecs_inout_kind_t_EcsInOutDefault;
    #[constant]
    pub const INOUT_MODE_FILTER:flecs::ecs_inout_kind_t = flecs::ecs_inout_kind_t_EcsInOutFilter;
    #[constant]
    pub const INOUT_MODE_NONE:flecs::ecs_inout_kind_t = flecs::ecs_inout_kind_t_EcsInOutNone;
    #[constant]
    pub const INOUT_MODE_INOUT:flecs::ecs_inout_kind_t = flecs::ecs_inout_kind_t_EcsInOut;
    #[constant]
    pub const INOUT_MODE_IN:flecs::ecs_inout_kind_t = flecs::ecs_inout_kind_t_EcsIn;
    #[constant]
    pub const INOUT_MODE_OUT:flecs::ecs_inout_kind_t = flecs::ecs_inout_kind_t_EcsOut;
    
    #[constant]
    pub const MAX_TERMS:usize = 32;

    #[func]
    fn _with(
        &mut self,
        component: Variant,
        inout: flecs::ecs_inout_kind_t,
    ) -> Gd<_GlecsBaseSystemBuilder> {
        self.with_oper(component, flecs::ecs_oper_kind_t_EcsAnd);
        self.last_term_mut().inout = inout as i16;
        self.to_gd()
    }

    #[func]
    fn _without(&mut self, component: Variant) -> Gd<_GlecsBaseSystemBuilder> {
        self.with_oper(component, flecs::ecs_oper_kind_t_EcsNot);
        self.to_gd()
    }

    #[func]
    fn _or_with(
        &mut self,
        component: Variant,
        inout: flecs::ecs_inout_kind_t,
    ) -> Gd<_GlecsBaseSystemBuilder> {
        self.with_oper(component, flecs::ecs_oper_kind_t_EcsOr);
        self.last_term_mut().inout = inout as i16;
        self.to_gd()
    }

    #[func]
    fn _maybe_with(
        &mut self,
        component: Variant,
        inout: flecs::ecs_inout_kind_t,
    ) -> Gd<_GlecsBaseSystemBuilder> {
        self.with_oper(component, flecs::ecs_oper_kind_t_EcsOptional);
        self.last_term_mut().inout = inout as i16;
        self.to_gd()
    }

    #[func]
    fn _all_from(&mut self, entity: Variant) -> Gd<_GlecsBaseSystemBuilder> {
        self.from_oper(entity, flecs::ecs_oper_kind_t_EcsAndFrom);
        self.to_gd()
    }

    #[func]
    fn _any_from(&mut self, entity: Variant) -> Gd<_GlecsBaseSystemBuilder> {
        self.from_oper(entity, flecs::ecs_oper_kind_t_EcsOrFrom);
        self.to_gd()
    }

    #[func]
    fn _none_from(&mut self, entity: Variant) -> Gd<_GlecsBaseSystemBuilder> {
        self.from_oper(entity, flecs::ecs_oper_kind_t_EcsNotFrom);
        self.to_gd()
    }

    #[func]
    fn _for_each(&mut self, callable:Callable) {
        self.on_build();
        let world = self.world.clone();

        match self.build_type {
            BuildType::System => _GlecsBaseWorld
                ::new_system_from_builder(world, self, callable),
            BuildType::Observer => _GlecsBaseWorld
                ::new_observer_from_builder(world, self, callable),
        }
    }

    #[func]
    fn _set_pipeline(&mut self, pipeline:Variant) -> Gd<_GlecsBaseSystemBuilder> {
        self.pipeline = pipeline;
        let gd = self.to_gd();
        gd
    }

    fn add_term_to_buffer(&mut self, term:flecs::ecs_term_t) {
        if self.term_count == Self::MAX_TERMS {
            panic!("Max terms reached. TODO: better msg")
        }
        self.description.terms[self.term_count] = term;
        self.term_count += 1;
    }

    fn with_oper(&mut self, component: Variant, oper:flecs::ecs_oper_kind_t) {
        // TODO: Add checks that scripts are indeed derived from components
        let comp_id = _GlecsBaseWorld
            ::_id_from_variant(self.world.clone(), component);
        
        let term = flecs::ecs_term_t {
            id: comp_id,
            oper: oper as i16,
            ..Default::default()
        };
        self.add_term_to_buffer(term);
    }

    fn from_oper(&mut self, entity: Variant, oper:flecs::ecs_oper_kind_t) {
        let entity_id = _GlecsBaseWorld::_id_from_variant(
            self.world.clone(),
            entity,
        );
        
        let term = flecs::ecs_term_t {
            id: entity_id,
            oper: oper as i16,
            ..Default::default()
        };

        self.add_term_to_buffer(term);
    }

    pub(crate) fn last_term(&self) -> &flecs::ecs_term_t {
        & self.description.terms[self.term_count-1]
    }
    
    pub(crate) fn last_term_mut(&mut self) -> &mut flecs::ecs_term_t {
        &mut self.description.terms[self.term_count-1]
    }

    fn on_build(&mut self) {
    }
}