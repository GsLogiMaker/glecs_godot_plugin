
use std::ffi::CString;
use std::ffi::CStr;

use flecs::EntityId;
use godot::prelude::*;

use crate::world::_GlecsBaseWorld;

#[derive(GodotClass)]
#[class(base=Object, no_init)]
pub struct _GlecsBindings {
	pub(crate) base: Base<Object>,
}
#[godot_api]
impl _GlecsBindings {
    #[func]
    pub(crate) fn new_id(world: Gd<_GlecsBaseWorld>) -> EntityId {
        Self::new_id_from_ref(&world.bind())
    }

    #[func]
    pub(crate) fn get_name(
        world: Gd<_GlecsBaseWorld>,
        entity: EntityId,
    ) -> GString {
        Self::get_name_from_ref(&world.bind(), entity)
    }

    #[func]
    pub(crate) fn set_name(
        world: Gd<_GlecsBaseWorld>,
        entity: EntityId,
        name: GString,
    ) -> EntityId {
        Self::set_name_from_ref(&world.bind(), entity, name)
    }

    #[func]
    pub(crate) fn _add_id(
        world: Gd<_GlecsBaseWorld>,
        entity: EntityId,
        id: EntityId,
    ) {
        Self::add_id_from_ref(&world.bind(), entity, id);
    }

    #[func]
    pub(crate) fn lookup(
        world: Gd<_GlecsBaseWorld>,
        name: GString,
    ) -> EntityId {
        Self::lookup_from_ref(&world.bind(), name)
    }

    #[func]
    pub(crate) fn _add_pair(
        world: Gd<_GlecsBaseWorld>,
        entity: EntityId,
        relation: EntityId,
        target: EntityId,
    ) {
        Self::add_pair_from_ref(&world.bind(), entity, relation, target);
    }

	#[func]
    pub(crate) fn _flecs_on_add() -> EntityId {
        unsafe { flecs::EcsOnAdd }
    }
    #[func]
    pub(crate) fn _flecs_on_remove() -> EntityId {
        unsafe { flecs::EcsOnRemove }
    }
    #[func]
    pub(crate) fn _flecs_on_set() -> EntityId {
        unsafe { flecs::EcsOnSet }
    }
    #[func]
    pub(crate) fn _flecs_un_set() -> EntityId {
        unsafe { flecs::EcsUnSet }
    }
    #[func]
    pub(crate) fn _flecs_monitor() -> EntityId {
        unsafe { flecs::EcsMonitor }
    }
    #[func]
    pub(crate) fn _flecs_on_delete() -> EntityId {
        unsafe { flecs::EcsOnDelete }
    }
    #[func]
    pub(crate) fn _flecs_on_table_create() -> EntityId {
        unsafe { flecs::EcsOnTableCreate }
    }
    #[func]
    pub(crate) fn _flecs_on_table_delete() -> EntityId {
        unsafe { flecs::EcsOnTableDelete }
    }
    #[func]
    pub(crate) fn _flecs_on_table_empty() -> EntityId {
        unsafe { flecs::EcsOnTableEmpty }
    }
    #[func]
    pub(crate) fn _flecs_on_table_fill() -> EntityId {
        unsafe { flecs::EcsOnTableFill }
    }
    #[func]
    pub(crate) fn _flecs_prefab() -> EntityId {
        unsafe { flecs::EcsPrefab }
    }
    #[func]
    pub(crate) fn _flecs_child_of() -> EntityId {
        unsafe { flecs::EcsChildOf }
    }
    #[func]
    pub(crate) fn _flecs_is_a() -> EntityId {
        unsafe { flecs::EcsIsA }
    }

    pub(crate) fn new_id_from_ref(world: &_GlecsBaseWorld) -> EntityId {
        unsafe { flecs::ecs_new_id(world.raw()) }
    }

    pub(crate) fn get_name_from_ref(
        world: &_GlecsBaseWorld,
        entity: EntityId,
    ) -> GString {
        GString::from(
            Self::get_name_cstr_from_ref(world, entity)
                .to_owned()
                .into_string()
                .unwrap()
        )
    }

    pub(crate) fn get_name_cstr_from_ref(
        world: &_GlecsBaseWorld,
        entity: EntityId,
    ) -> &CStr {
        let name_ptr = unsafe { flecs::ecs_get_name(
            world.raw(),
            entity,
        ) };
        if name_ptr == std::ptr::null() {
            return cstr::cstr!(b"");
        }
        let name_cstr = unsafe { CStr::from_ptr(name_ptr) };
        
        name_cstr
    }
    
    pub(crate) fn set_name_from_ref(
        world: &_GlecsBaseWorld,
        entity: EntityId,
        name: GString,
    ) -> EntityId {
        unsafe { flecs::ecs_set_name(
            world.raw(),
            entity,
            gstring_to_cstring(name).as_ptr(),
        ) }
    }

    pub(crate) fn add_id_from_ref(
        world: &_GlecsBaseWorld,
        entity: EntityId,
        id: EntityId,
    ) {
        unsafe { flecs::ecs_add_id(
            world.raw(),
            entity,
            id,
        ) };
    }

    pub(crate) fn lookup_from_ref(
        world: &_GlecsBaseWorld,
        name: GString,
    ) -> EntityId {
        let path = gstring_to_cstring(name);
        let sep = CString::new("/").unwrap();
        let prefix = CString::new("").unwrap();
        let got = unsafe {
            flecs::ecs_lookup_path_w_sep(
                world.raw(),
                0,
                path.as_ptr(),
                sep.as_ptr(),
                prefix.as_ptr(),
                false,
            )
        };
        if false {dbg!();}
        got
    }

    pub(crate) fn add_pair_from_ref(
        world: &_GlecsBaseWorld,
        entity: EntityId,
        relation: EntityId,
        target: EntityId,
    ) {
        Self::add_id_from_ref(
            world,
            entity,
            flecs::ecs_pair(relation, target),
        );
    }
}

fn gstring_to_cstring(text: GString) -> CString {
    unsafe { CString::from_vec_unchecked(Vec::from(text.to_string())) }
}