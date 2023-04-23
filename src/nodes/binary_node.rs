use bevy::{math::vec2, prelude::*};
use bevy_mod_picking::PickableBundle;
use bevy_prototype_lyon::prelude::*;

use super::Node;
use crate::{NodeWrapper, COLOR_NODE_BG};

#[derive(Debug, Default)]
pub struct NodeSocket<S> {
	// Position relative to the nodes origin
	pos: Vec2,
	state: Option<S>,
}

impl<S> NodeSocket<S> {
	pub fn new(pos: Vec2, state: Option<S>) -> Self {
		Self { pos, state }
	}
}

#[derive(Debug)]
pub struct BinaryNode {
	pos: Vec2,
	input_1: NodeSocket<bool>,
	input_2: NodeSocket<bool>,
	output: NodeSocket<bool>,
}

impl BinaryNode {
	pub fn new(pos: Vec2) -> Self {
		let height = 3.0;
		let width = 5.0;

		Self {
			pos,
			input_1: NodeSocket::new(vec2(pos.x, pos.y), None),
			input_2: NodeSocket::new(vec2(pos.x, pos.y - 1.0 * height), None),
			output: NodeSocket::new(vec2(pos.x + width, pos.y - 0.5 * height), None),
		}
	}
}

impl Node for BinaryNode {
	fn init(&self, commands: &mut Commands, index: crate::NodeIndex) {
		let socket = |pos| {
			(
				NodeWrapper { index },
				PickableBundle::default(),
				ShapeBundle {
					path: GeometryBuilder::build_as(&shapes::Circle {
						radius: 1.0,
						center: pos,
					}),
					..default()
				},
				Fill {
					options: FillOptions::tolerance(0.05),
					color: COLOR_NODE_BG,
				},
			)
		};

		commands.spawn(socket(self.input_1.pos));
		commands.spawn(socket(self.input_2.pos));
		commands.spawn(socket(self.output.pos));
	}

	fn render(&self, _path: &mut Path) {}
}
