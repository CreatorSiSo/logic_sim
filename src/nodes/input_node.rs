use bevy::prelude::*;
use bevy_mod_picking::{NoDeselect, PickableBundle};
use bevy_prototype_lyon::prelude::*;

use super::Node;
use crate::{NodeIndex, NodeWrapper, COLOR_NODE_BG};

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

impl Node for InputNode {
	fn init(&self, commands: &mut Commands, index: NodeIndex) {
		commands.spawn((
			NodeWrapper { index },
			NoDeselect,
			PickableBundle::default(),
			ShapeBundle {
				path: GeometryBuilder::build_as(&shapes::Circle {
					radius: 1.0,
					center: self.pos,
				}),
				..default()
			},
			Fill {
				options: FillOptions::tolerance(0.05),
				color: COLOR_NODE_BG,
			},
		));
	}

	fn render(&self, _path: &mut Path) {}
}
