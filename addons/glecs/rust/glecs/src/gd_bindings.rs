
use flecs::EntityId;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Object, no_init)]
pub struct _GlecsBindings {
	pub(crate) base: Base<Object>,
}
#[godot_api]
impl _GlecsBindings {
	#[func]
    fn _flecs_on_add() -> EntityId {
        unsafe { flecs::EcsOnAdd }
    }
    #[func]
    fn _flecs_on_remove() -> EntityId {
        unsafe { flecs::EcsOnRemove }
    }
    #[func]
    fn _flecs_on_set() -> EntityId {
        unsafe { flecs::EcsOnSet }
    }
    #[func]
    fn _flecs_un_set() -> EntityId {
        unsafe { flecs::EcsUnSet }
    }
    #[func]
    fn _flecs_monitor() -> EntityId {
        unsafe { flecs::EcsMonitor }
    }
    #[func]
    fn _flecs_on_delete() -> EntityId {
        unsafe { flecs::EcsOnDelete }
    }
    #[func]
    fn _flecs_on_table_create() -> EntityId {
        unsafe { flecs::EcsOnTableCreate }
    }
    #[func]
    fn _flecs_on_table_delete() -> EntityId {
        unsafe { flecs::EcsOnTableDelete }
    }
    #[func]
    fn _flecs_on_table_empty() -> EntityId {
        unsafe { flecs::EcsOnTableEmpty }
    }
    #[func]
    fn _flecs_on_table_fill() -> EntityId {
        unsafe { flecs::EcsOnTableFill }
    }
    #[func]
    fn _flecs_prefab() -> EntityId {
        unsafe { flecs::EcsPrefab }
    }
    #[func]
    fn _flecs_is_a() -> EntityId {
        unsafe { flecs::EcsIsA }
    }
}