mod key;
mod operations;

pub use key::Key;
pub use operations::{Operation, Operations};
pub type KeybindingsRaw = HashMap<String, Vec<String>>;

use crate::ui::Event;
use anyhow::{bail, Result};
use std::{collections::HashMap, sync::mpsc::Sender};

pub struct Keybindings {
	keybindings: HashMap<Key, Operations>,
}

impl Keybindings {
	pub fn add_event_tx(&mut self, event_tx: &Sender<Event>) {
		for ops in self.keybindings.values_mut() {
			ops.add_tx(event_tx);
		}
	}

	pub fn get_operations(&self, key: &Key) -> Option<&Operations> {
		self.keybindings.get(key)
	}
}

impl TryFrom<KeybindingsRaw> for Keybindings {
	type Error = anyhow::Error;
	fn try_from(value: KeybindingsRaw) -> Result<Self, Self::Error> {
		let keybindings = value
			.into_iter()
			.map(|(key, ops)| Ok((key.parse()?, Operations::from_vec(ops)?)))
			.collect::<Result<_>>()?;
		Ok(Self { keybindings })
	}
}

// TODO: return (&str, &str), deal with lifetime
// TODO: replace with nom
pub fn parse_str(s: &str) -> Result<(String, Vec<String>)> {
	let Some((key, operations)) = s.split_once(':') else {
		bail!("invalid format: expected \"KEY:OP[+OP]*\", found \"{}\"", s);
	};

	Ok((
		key.to_string(),
		operations
			.split('+')
			.map(|op| op.trim().to_owned())
			.collect(),
	))
}

// new and old have same key => keep new value
pub fn merge_raw(
	new_opt: Option<KeybindingsRaw>,
	old_opt: Option<KeybindingsRaw>,
) -> Option<KeybindingsRaw> {
	match new_opt {
		Some(new) => match old_opt {
			Some(old) => {
				// TODO: borrow old as mutable and avoid clone
				let mut merged = old.clone();
				merged.extend(new);
				Some(merged)
			}
			None => Some(new),
		},
		None => old_opt,
	}
}
