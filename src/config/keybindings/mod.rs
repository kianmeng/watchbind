mod key;
mod operations;

pub use key::Key;
pub use operations::{Operation, Operations};

use crate::ui::Event;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::{collections::HashMap, sync::mpsc::Sender};

pub struct Keybindings(HashMap<Key, Operations>);

impl Keybindings {
	pub fn add_event_tx(&mut self, event_tx: &Sender<Event>) {
		for ops in self.0.values_mut() {
			ops.add_tx(event_tx);
		}
	}

	pub fn get_operations(&self, key: &Key) -> Option<&Operations> {
		self.0.get(key)
	}
}

impl TryFrom<RawKeybindings> for Keybindings {
	type Error = anyhow::Error;
	fn try_from(value: RawKeybindings) -> Result<Self, Self::Error> {
		let keybindings = value
			.0
			.into_iter()
			.map(|(key, ops)| Ok((key.parse()?, Operations::from_vec(ops)?)))
			.collect::<Result<_>>()?;
		Ok(Self(keybindings))
	}
}

// TODO: remove once clap supports parsing directly into HashMap
pub type ClapKeybindings = Vec<(String, Vec<String>)>;

#[derive(Deserialize)]
pub struct RawKeybindings(HashMap<String, Vec<String>>);

impl RawKeybindings {
	pub fn merge(new_opt: Option<Self>, old_opt: Option<Self>) -> Option<Self> {
		match new_opt {
			Some(new) => match old_opt {
				Some(old) => {
					// new and old have same key => keep new value
					let mut merged = old.0;
					merged.extend(new.0);
					Some(RawKeybindings(merged))
				}
				None => Some(new),
			},
			None => old_opt,
		}
	}
}

impl From<ClapKeybindings> for RawKeybindings {
	fn from(clap: ClapKeybindings) -> Self {
		Self(clap.into_iter().collect())
	}
}

// TODO: replace with nom
// TODO: parse to Vec<Keybinding> and provide from_str for keybinding
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
