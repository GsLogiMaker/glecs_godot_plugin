
use std::alloc::Layout;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::ffi::c_void;
use std::pin::Pin;
use std::rc::Rc;

use flecs::EntityId;
use flecs::Iter;
use flecs::TermBuilder;
use flecs::World as FlWorld;
use godot::engine::notify::NodeNotification;
use godot::engine::Script;
use godot::prelude::*;

use crate::component::_BaseGEComponent;
use crate::component_definitions::ComponentDefinitions;
use crate::component_definitions::ComponentDefinitionsMapKey;
use crate::component_definitions::ComponetDefinition;
use crate::component_definitions::ComponetProperty;
use crate::entity::_BaseGEEntity;
use crate::TYPE_SIZES;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct _BaseGEWorld {
    #[base] pub(crate) node: Base<Node>,
    pub(crate) world: FlWorld,
    component_definitions: ComponentDefinitions,
    system_contexts: LinkedList<Pin<Box<ScriptSystemContext>>>,
    gd_entity_map: HashMap<EntityId, Gd<_BaseGEEntity>>,
	deleting:bool
}
#[godot_api]
impl _BaseGEWorld {
    #[func]
    fn _world_process(&mut self, delta:f32) {
        self.world.progress(delta);
    }

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

    /// Creates a new entity in the world.
    #[func]
    fn _new_entity(
        &mut self,
        with_components:Array<Gd<Script>>,
    ) -> Gd<_BaseGEEntity> {
        let mut entity = self.world.entity();
        let mut i = 0;
        while i != with_components.len() {
            let mut script = with_components.get(i);

            let comp_def = self
                .get_or_add_component(&script);
            entity = entity.add_id(comp_def.flecs_id);

            let data = entity.get_mut_dynamic(&comp_def.name.to_string());

            i += 1;

            // Initialize component properties
            // TODO: Initialize properties in deterministic order
            for property_name in comp_def.parameters.keys() {
                // TODO: Get default values of properties
                let default_value = script
                    .get_property_default_value(property_name.clone());
                _BaseGEComponent::_initialize_property(
                    data,
                    comp_def.as_ref(),
                    property_name.clone(),
                    default_value,
                );
            }
        }

        let gd_entity = Gd::from_init_fn(|base| {
            _BaseGEEntity {
                base,
                world: self.to_gd(),
                id: entity.id(),
				world_deletion: false,
                gd_components_map: Default::default(),
            }
        });
        self.gd_entity_map.insert(entity.id(), gd_entity.clone());
        
        gd_entity
    }

    // Defines a new system to be run in the world.
    #[func]
    fn _add_system(&mut self, callable: Callable, terms: Array<Gd<Script>>) {
        // Create term list
        let mut term_ids = vec![];
        for i in 0..terms.len() {
            let script = terms.get(i);
			
            let comp_def = self
				.get_or_add_component(&script);
            term_ids.push(comp_def.flecs_id);
        }

        // Create component accesses
        let mut system_args = array![];
        let mut tarm_accesses: Vec<Gd<_BaseGEComponent>> = vec![];
        for term_i in 0..terms.len() as usize {
            let term_script = terms.get(term_i).clone();
            let mut compopnent_access = Gd
                ::<_BaseGEComponent>
                ::from_init_fn(|base| {
                    _BaseGEComponent {
                        base,
                        flecs_id: term_ids[term_i],
                        data: &mut [],
                        component_definition: self
                            .get_or_add_component(&term_script),
                    }
                });
            compopnent_access.set_script(term_script.to_variant());
            system_args.push(compopnent_access.to_variant());
            tarm_accesses.push(compopnent_access);
        }
        let term_args_fast = tarm_accesses
            .into_boxed_slice();

        // Create contex
        self.system_contexts.push_back(Pin::new(Box::new(
            ScriptSystemContext {
                system_args: system_args,
                term_accesses: term_args_fast,
                callable: callable.clone(),
                terms: terms,
                world: self.to_gd(),
            }
        )));
        let context_ptr:*mut Pin<Box<ScriptSystemContext>> = self
            .system_contexts
            .back_mut()
            .unwrap();

        // Create system
        let mut sys = self.world
            .system()
            .context_ptr(context_ptr as *mut c_void);
        for id in term_ids.iter() {
            sys = sys.term_dynamic(*id);
        }

        // System body
        sys.iter(Self::system_iteration);
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
        let value = ComponentDefinitionsMapKey
            ::from(key)
            .get_value(&self.component_definitions);
        match value {
            Some(value) => value,
            None => {
                let def = ComponetDefinition::new(
                    key.clone(),
                    self,
                );
                self.component_definitions.insert(def)
            }
        }
    }

    pub(crate) fn layout_from_properties(
        parameters: &HashMap<StringName, ComponetProperty>,
    ) -> Layout {
        let mut size = 0;
        for (_name, property) in parameters {
            size += TYPE_SIZES[property.gd_type_id as usize];
        }
        Layout::from_size_align(size, 8).unwrap()
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

	// Get context
	pub(crate) fn system_iteration(iter:&Iter) {
		let context = unsafe {
			(iter as *const Iter)
				.cast_mut()
				.as_mut()
				.unwrap()
				.get_context_mut::<Pin<Box<ScriptSystemContext>>>()
		};

		for entity_index in 0..(iter.count() as usize) {
			// Create components arguments
			for field_i in 1i32..(iter.field_count()+1) {
				let mut column = iter
					.field_dynamic(field_i+1);
				let data:*mut [u8] = column.get_mut(entity_index);

				context.term_accesses[field_i as usize]
					.bind_mut()
					.data = data;
			}
			
			let _result = context.callable.callv(
				context.system_args.clone()
			);
		}
	}
}

#[godot_api]
impl INode for _BaseGEWorld {
    fn init(node: Base<Node>) -> Self {
        let world = FlWorld::new();
        Self {
            node,
            world: world,
            component_definitions: Default::default(),
            system_contexts: Default::default(),
            gd_entity_map: Default::default(),
			deleting: false,
        }
    }

	fn on_notification(&mut self, what: NodeNotification) {
        match what {
            NodeNotification::Predelete => {
                self.on_free()
            },
            _ => {},
        }
    }

    // fn physics_process(&mut self, delta:f64) {
    //     self.world.progress(delta as f32);
    // }
}

#[derive(Debug, Clone)]
pub(crate) struct ScriptSystemContext {
    callable: Callable,
    terms: Array<Gd<Script>>,
    /// The arguments passed to the system.
    system_args: Array<Variant>,
    /// Holds the accesses stored in `sysatem_args` for quicker access.
    term_accesses: Box<[Gd<_BaseGEComponent>]>,
    world: Gd<_BaseGEWorld>,
}