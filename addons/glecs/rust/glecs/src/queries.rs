
use flecs::*;
use godot::engine::Script;
use godot::prelude::*;

use crate::gd_bindings::_GlecsQueries;
use crate::world::_GlecsBaseWorld;
use crate::Int;

pub(crate) fn load_system_builder_script() -> Variant {
    load::<Script>("res://addons/glecs/gd/system_builder.gd")
        .to_variant()
}

#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct _GlecsBaseQueryBuilder {
	pub(crate) base: Base<RefCounted>,
    pub(crate) desc: ecs_query_desc_t,
    pub(crate) term_count: usize,
    pub(crate) world: Gd<_GlecsBaseWorld>,
    pub(crate) each_callback: Option<Callable>,
    pub(crate) build_callback: Box<dyn FnMut()>,
}
#[godot_api]
impl _GlecsBaseQueryBuilder {
    #[func(gd_self)]
    pub fn _with_term(this: Gd<Self>, id: Variant) -> Gd<Self> {
        Self::add_term(this.clone(), id, _GlecsQueries::OPER_AND);
        this
    }

    #[func(gd_self)]
    pub fn _without_term(this: Gd<Self>, id: Variant) -> Gd<Self> {
        Self::add_term(this.clone(), id, _GlecsQueries::OPER_NOT);
        this
    }

    #[func(gd_self)]
    pub fn _or_with_term(this: Gd<Self>, id: Variant) -> Gd<Self> {
        Self::add_term(this.clone(), id, _GlecsQueries::OPER_OR);
        this
    }

    #[func(gd_self)]
    pub fn _with_all_terms_from(this: Gd<Self>, id: Variant) -> Gd<Self> {
        Self::add_term(this.clone(), id, _GlecsQueries::OPER_AND_FROM);
        this
    }

    #[func(gd_self)]
    pub fn _with_any_term_from(this: Gd<Self>, id: Variant) -> Gd<Self> {
        Self::add_term(this.clone(), id, _GlecsQueries::OPER_OR_FROM);
        this
    }

    #[func(gd_self)]
    pub fn _with_no_terms_from(this: Gd<Self>, id: Variant) -> Gd<Self> {
        Self::add_term(this.clone(), id, _GlecsQueries::OPER_NOT_FROM);
        this
    }

    #[func(gd_self)]
    pub fn for_each(mut this:Gd<Self>, callable:Callable) {
        let mut this_must = this.bind_mut();
        this_must.each_callback = Some(callable);
        (this_must.build_callback)();
    }

    pub(crate) fn add_term(this:Gd<Self>, id:Variant, oper:Int){
        let world = this.bind().world.clone();
        let term_id = _GlecsBaseWorld::_id_from_variant(world, id);
        
        _GlecsQueries::add_term(this.clone(), term_id);
        _GlecsQueries::set_term_oper(this.clone(), oper);
    }

    // Returns a slice of set terms.
    pub(crate) fn get_terms(&self) -> &[ecs_term_t] {
        self.desc.terms.split_at(self.term_count).0
    }
}
#[godot_api]
impl IRefCounted for _GlecsBaseQueryBuilder {
    fn init(base:Base<RefCounted>) -> Self {
        Self {
            base,
            desc: Default::default(),
            term_count: 0,
            world: _GlecsBaseWorld::_get_global(),
            each_callback: None,
            build_callback: Box::new(|| {}),
        }
    }
}