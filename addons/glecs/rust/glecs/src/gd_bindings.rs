
use std::ffi::c_char;
use std::ffi::CString;
use std::ffi::CStr;

use flecs::*;
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
    pub(crate) fn emit_event(world: Gd<_GlecsBaseWorld>, event:EntityId, to_entity:EntityId, components:PackedInt64Array) {
        let world_raw = world.bind().raw();
        let mut event_desc = ecs_event_desc_t {
            event: event,
            ids: &ecs_type_t {
                array: (&mut (components[0] as EntityId)) as *mut EntityId,
                count: components.len() as i32,
            },
            entity: to_entity,
            ..Default::default()
        };
        unsafe { ecs_emit(world_raw, &mut event_desc) };
    }

    #[func]
    pub(crate) fn new_id(world: Gd<_GlecsBaseWorld>) -> EntityId {
        Self::new_id_from_ref(&world.bind())
    }

    #[func]
    pub(crate) fn module_init(
        world: Gd<_GlecsBaseWorld>,
        name: GString,
        source_id: EntityId,
    ) -> EntityId {
        Self::module_init_from_ref(&world.bind(), name, source_id)
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
    pub(crate) fn pair(
        first: EntityId,
        second: EntityId,
    ) -> EntityId {
        unsafe { ecs_make_pair(first, second) }
    }

    #[func]
    pub(crate) fn pair_first(
        pair: EntityId,
    ) -> EntityId {
        ((pair & ECS_COMPONENT_MASK) >> 32) as u32 as EntityId
    }

    #[func]
    pub(crate) fn pair_second(
        pair: EntityId,
    ) -> EntityId {
        pair as u32 as EntityId
    }

    #[func]
    pub(crate) fn id_is_alive(
        world: Gd<_GlecsBaseWorld>,
        id: EntityId,
    ) -> bool {
        if !world.is_instance_valid() {
            // World is deleted
            return false
        }

        if Self::id_is_pair(id) {
            let first_id = Self::pair_first(id);
            let second_id = Self::pair_second(id);
            let first_alive = unsafe {ecs_is_alive(world.bind().raw(), first_id)};
            let second_alive = unsafe {ecs_is_alive(world.bind().raw(), second_id)};

            return first_alive && second_alive;
        }

        unsafe { ecs_is_alive(world.bind().raw(), id) }
    }

    #[func]
    pub(crate) fn id_is_pair(
        entity: EntityId,
    ) -> bool {
        unsafe { ecs_id_is_pair(entity) }
    }

    #[func]
    pub(crate) fn has_id(
        world: Gd<_GlecsBaseWorld>,
        entity: EntityId,
        id: EntityId,
    ) -> bool {
        unsafe { ecs_has_id(world.bind().raw(), entity, id) }
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
    pub(crate) fn lookup_child(
        world: Gd<_GlecsBaseWorld>,
        parent: EntityId,
        name: GString,
    ) -> EntityId {
        Self::lookup_child_from_ref(&world.bind(), parent, name)
    }

    #[func]
    pub(crate) fn add_pair(
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
        unsafe { flecs::ecs_new(world.raw()) }
    }

    pub(crate) fn module_init_from_ref(
        world: &_GlecsBaseWorld,
        name: GString,
        source_id: EntityId,
    ) -> EntityId {
        let mut desc = flecs::ecs_component_desc_t::default();
        desc.entity = source_id;
        unsafe { ecs_module_init(
            world.raw(),
            gstring_to_cstring(name).as_ptr(),
            &desc,
        ) }
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
    
    pub(crate) fn set_name_c(
        world: &_GlecsBaseWorld,
        entity: EntityId,
        name: CString,
    ) -> EntityId {
        unsafe { flecs::ecs_set_name(
            world.raw(),
            entity,
            name.as_ptr(),
        ) }
    }
    
    pub(crate) fn set_name_from_ref(
        world: &_GlecsBaseWorld,
        entity: EntityId,
        name: GString,
    ) -> EntityId {
        Self::set_name_c(world, entity, gstring_to_cstring(name))
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
        Self::lookup_c(world, path.as_ptr())
    }

    pub(crate) fn lookup_c(
        world: &_GlecsBaseWorld,
        name: *const c_char,
    ) -> EntityId {
        let path = name;
        let sep = CString::new("/").unwrap();
        let prefix = CString::new("").unwrap();
        let got = unsafe {
            flecs::ecs_lookup_path_w_sep(
                world.raw(),
                0,
                path,
                sep.as_ptr(),
                prefix.as_ptr(),
                false,
            )
        };
        
        got
    }

    pub(crate) fn lookup_child_from_ref(
        world: &_GlecsBaseWorld,
        parent: EntityId,
        name: GString,
    ) -> EntityId {
        let path = gstring_to_cstring(name);
        let got = unsafe {
            flecs::ecs_lookup_child(
                world.raw(),
                parent,
                path.as_ptr(),
            )
        };
        
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
            Self::pair(relation, target),
        );
    }
}

fn gstring_to_cstring(text: GString) -> CString {
    unsafe { CString::from_vec_unchecked(Vec::from(text.to_string())) }
}