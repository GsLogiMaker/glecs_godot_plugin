
use flecs::EntityId;
use godot::engine::Script;
use godot::prelude::*;

use crate::component::_BaseGEComponent;
use crate::entity::EcsIdAttachment;
use crate::world::_BaseGEWorld;

pub(crate) const PREFAB_COMPONENTS:&str = "COMPONENTS";

#[derive(GodotClass, Debug)]
#[class(base=RefCounted)]
pub struct _BaseGEPrefab {
    #[base] pub(crate) base: Base<RefCounted>,
    /// The world this entity is from.
    pub(crate) world: Gd<_BaseGEWorld>,
    /// The Flecs ID of this prefab.
    pub(crate) flecs_id: EntityId,
}
#[godot_api]
impl _BaseGEPrefab {
	#[func]
	fn add_component(
		&mut self,
		component:Gd<Script>,
	) -> Option<Gd<_BaseGEComponent>>{
		EcsIdAttachment::add_component(self, component)
	}
}
impl EcsIdAttachment for _BaseGEPrefab {
    fn get_world(&self) -> Gd<_BaseGEWorld> {
        self.world.clone()
    }

    fn get_flecs_id(&self) -> EntityId {
        self.flecs_id
    }
}


#[derive(Debug, Clone)]
pub(crate) struct PrefabDefinition {
    pub(crate) script: Gd<Script>,
    pub(crate) flecs_id: EntityId,
} impl PrefabDefinition {
    pub(crate) fn new(
        mut script:Gd<Script>,
        world:&mut _BaseGEWorld,
    ) -> PrefabDefinition {
        let prefab_entt = world.world
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
                world.get_or_add_component(&component).flecs_id
            );
        }

        PrefabDefinition {
            script: script,
            flecs_id: prefab_entt.id(),
        }
    }
}