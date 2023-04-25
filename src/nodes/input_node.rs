use bevy::prelude::*;
use bevy_mod_picking::{NoDeselect, PickableBundle};
use bevy_prototype_lyon::prelude::*;

use super::UiElement;
use crate::{color, NodeId, NodeIndex};

#[derive(Debug)]
pub struct InputNode {
	pub state: bool,
	pub pos: Vec2,
}

impl InputNode {
	pub fn new(state: bool, pos: Vec2) -> Self {
		Self { state, pos }
	}
}

impl UiElement for InputNode {
	fn init(&self, commands: &mut Commands, index: NodeIndex) {
		commands.spawn((
			NodeId(index),
			NoDeselect,
			PickableBundle::default(),
			ShapeBundle {
				path: GeometryBuilder::build_as(&shapes::Circle {
					radius: 5.0,
					center: self.pos,
				}),
				..default()
			},
			Fill {
				options: FillOptions::tolerance(0.05),
				color: color::NODE_SOCKET,
			},
			Stroke::new(color::NODE_SOCKET, 1.5),
		));
	}

	fn render(&self, _path: &mut Path) {}
}
