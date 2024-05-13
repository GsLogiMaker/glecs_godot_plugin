
use godot::engine::Script;
use godot::prelude::*;

pub(crate) fn script_inherets(script: Gd<Script>, inherets: Gd<Script>) -> bool {
	let mut s = script;
	while s != inherets {
		let maybe_s = s.get_base_script();
		
		if maybe_s.is_none() {
			return false
		}

		s = maybe_s.unwrap()
	}

	true
}