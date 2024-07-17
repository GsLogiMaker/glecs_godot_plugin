
use flecs::EntityId;
use godot::engine::Script;
use godot::prelude::*;

use crate::component::_GlecsBaseComponent;
use crate::entity::EntityLike;
use crate::gd_bindings::_GlecsBindings;
use crate::world::_GlecsBaseWorld;

pub(crate) const PREFAB_COMPONENTS:&str = "COMPONENTS";

#[derive(GodotClass, Debug)]
#[class(base=RefCounted, no_init)]
pub struct _GlecsPrefab {
    pub(crate) base: Base<RefCounted>,
    /// The world this entity is from.
    pub(crate) world: Gd<_GlecsBaseWorld>,
    /// The Flecs ID of this prefab.
    pub(crate) flecs_id: EntityId,
}
#[godot_api]
impl _GlecsPrefab {
}


#[derive(Debug, Clone)]
pub(crate) struct PrefabDefinition {
    pub(crate) script: Gd<Script>,
    pub(crate) flecs_id: EntityId,
} impl PrefabDefinition {
    pub(crate) fn new(
        mut script:Gd<Script>,
        world:&mut _GlecsBaseWorld,
    ) -> PrefabDefinition {
        let prefab_entt = _GlecsBindings::lookup_from_ref(
            world,
            script.instance_id().to_string().into(),
        );

        let componets = script.get_script_constant_map()
            .get(StringName::from(PREFAB_COMPONENTS))
            .unwrap_or_else(|| {Array::<Variant>::default().to_variant()})
            .try_to::<Array<Variant>>()
            .unwrap_or_default();

        for component in componets.iter_shared() {
            let Ok(component_script) = component
                .try_to::<Gd<Script>>()
                else {continue};
                
            let component = world.get_or_add_component(component_script);
            _GlecsBindings::add_id_from_ref(
                world,
                prefab_entt,
                component,
            );
        }

        PrefabDefinition {
            script: script,
            flecs_id: prefab_entt,
        }
    }
}