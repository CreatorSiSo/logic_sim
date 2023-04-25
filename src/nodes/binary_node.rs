use bevy::{math::vec2, prelude::*};
use bevy_mod_picking::PickableBundle;
use bevy_prototype_lyon::prelude::*;

use super::{rounded_rect, z_transform, NodeSocket, UiElement};
use crate::{color, NodeId};

#[derive(Debug)]
pub struct BinaryNode {
	center: Vec2,
	width: f32,
	height: f32,
	input_1: NodeSocket<bool>,
	input_2: NodeSocket<bool>,
	output: NodeSocket<bool>,
}

impl BinaryNode {
	pub fn new(center: Vec2, width: f32, height: f32) -> Self {
		let half_width = width / 2.0;
		Self {
			center,
			width,
			height,
			input_1: NodeSocket::new(1, vec2(center.x - half_width, center.y + 1.0), None),
			input_2: NodeSocket::new(2, vec2(center.x - half_width, center.y - 1.0), None),
			output: NodeSocket::new(3, vec2(center.x + half_width, center.y), None),
		}
	}
}

impl UiElement for BinaryNode {
	fn init(&self, commands: &mut Commands, node_index: crate::NodeIndex) {
		self.input_1.init(commands, node_index);
		self.input_2.init(commands, node_index);
		self.output.init(commands, node_index);

		commands.spawn((
			NodeId(node_index),
			PickableBundle::default(),
			ShapeBundle {
				path: rounded_rect(self.center, self.width, self.height, 0.5),
				transform: z_transform(node_index, 0.0),
				..default()
			},
			Fill {
				options: FillOptions::tolerance(0.05),
				color: color::NODE,
			},
			Stroke::new(color::NODE_SOCKET, 3.0),
		));
	}

	fn render(&self, _path: &mut Path) {}
}
