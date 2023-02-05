mod line;

use crate::config::Styles;
use itertools::izip;
use line::Line;
use tui::{
	backend::Backend,
	layout::Constraint,
	widgets::{Row, Table, TableState},
	Frame,
};

const FIRST_INDEX: usize = 0;

// type Line = (String, Style);

pub struct State {
	lines: Vec<Line>,
	selected: Vec<bool>,
	styles: Styles,
	cursor: Option<usize>,
	// TODO: deprecate in future
	table_state: TableState,
}

impl State {
	pub fn new(styles: Styles) -> Self {
		Self {
			lines: vec![],
			selected: vec![],
			styles,
			cursor: None,
			table_state: TableState::default(),
		}
	}

	pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
		// TODO: do as much as possible in set_lines to improve performance
		let rows: Vec<Row> = izip!(self.lines.iter(), self.selected.iter())
			.map(|(line, &selected)| {
				let style = if selected {
					self.styles.selected
				} else {
					self.styles.line
				};
				line.draw(style)
			})
			.collect();

		let table = Table::new(rows)
			.widths(&[Constraint::Length(1), Constraint::Percentage(100)])
			.column_spacing(0);

		frame.render_stateful_widget(table, frame.size(), &mut self.table_state);
	}

	pub fn set_lines(&mut self, lines: Vec<String>) {
		self.selected.resize(lines.len(), false);
		self.lines = lines
			.into_iter()
			.map(|line| Line::new(line, self.styles.line))
			.collect();
		self.cursor_calibrate();
	}

	fn cursor_position(&mut self) -> Option<usize> {
		self.cursor
	}

	fn cursor_move(&mut self, index: isize) {
		let old = self.cursor_position();
		let new = if self.lines.is_empty() {
			None
		} else {
			let first = FIRST_INDEX as isize;
			let last = self.last_index() as isize;
			Some(index.clamp(first, last) as usize)
		};

		self.cursor = new;
		self.table_state.select(self.cursor);
		self.cursor_adjust_style(old, new);
	}

	fn cursor_calibrate(&mut self) {
		match self.cursor_position() {
			None => self.first(),
			Some(i) => self.cursor_move(i as isize),
		};
	}

	fn cursor_adjust_style(&mut self, old: Option<usize>, new: Option<usize>) {
		if let Some(old_index) = old {
			if let Some(old_cursor) = self.lines.get_mut(old_index) {
				old_cursor.set_style(self.styles.line);
			}
		}
		if let Some(new_index) = new {
			if let Some(new_cursor) = self.lines.get_mut(new_index) {
				new_cursor.set_style(self.styles.cursor);
			}
		}
	}

	fn get_cursor_line(&mut self) -> Option<String> {
		if let Some(i) = self.cursor_position() {
			if let Some(line) = self.lines.get(i) {
				return Some(line.get());
			}
		}
		None
	}

	pub fn get_selected_lines(&mut self) -> Option<String> {
		let lines: String = izip!(self.lines.iter(), self.selected.iter())
			.filter_map(|(line, &selected)| selected.then(|| line.get()))
			.collect::<Vec<String>>()
			.join("\n");

		if lines.is_empty() {
			self.get_cursor_line()
		} else {
			Some(lines)
		}
	}

	pub fn down(&mut self, steps: usize) {
		if let Some(i) = self.cursor_position() {
			self.cursor_move(i as isize + steps as isize);
		}
	}

	pub fn up(&mut self, steps: usize) {
		if let Some(i) = self.cursor_position() {
			self.cursor_move(i as isize - steps as isize);
		}
	}

	pub fn first(&mut self) {
		self.cursor_move(FIRST_INDEX as isize);
	}

	pub fn last(&mut self) {
		self.cursor_move(self.last_index() as isize);
	}

	pub fn select(&mut self) {
		if let Some(i) = self.cursor_position() {
			self.selected[i] = true;
		}
	}

	pub fn unselect(&mut self) {
		if let Some(i) = self.cursor_position() {
			self.selected[i] = false;
		}
	}

	pub fn select_toggle(&mut self) {
		if let Some(i) = self.cursor_position() {
			self.selected[i] = !self.selected[i];
		}
	}

	pub fn select_all(&mut self) {
		self.selected.fill(true);
	}

	pub fn unselect_all(&mut self) {
		self.selected.fill(false);
	}

	fn last_index(&self) -> usize {
		if self.lines.is_empty() {
			0
		} else {
			self.lines.len() - 1
		}
	}
}