use bevy::{math::vec2, prelude::*};
use bevy_mod_picking::PickableBundle;
use bevy_prototype_lyon::prelude::*;

use super::Node;
use crate::{color, NodeId, Z_TRANSFORM_NODE, Z_TRANSFORM_NODE_SOCKET};

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
			input_1: NodeSocket::new(vec2(center.x - half_width, center.y + 1.0), None),
			input_2: NodeSocket::new(vec2(center.x - half_width, center.y - 1.0), None),
			output: NodeSocket::new(vec2(center.x + half_width, center.y), None),
		}
	}
}

impl Node for BinaryNode {
	fn init(&self, commands: &mut Commands, index: crate::NodeIndex) {
		let socket = |pos| {
			(
				NodeId(index),
				PickableBundle::default(),
				ShapeBundle {
					path: GeometryBuilder::build_as(&shapes::Circle {
						radius: 5.0,
						center: pos,
					}),
					transform: Z_TRANSFORM_NODE_SOCKET,
					..default()
				},
				Fill {
					options: FillOptions::tolerance(0.05),
					color: color::NODE_SOCKET,
				},
				Stroke {
					options: StrokeOptions::DEFAULT.with_line_width(1.5),
					color: color::NODE_SOCKET,
				},
			)
		};
		commands.spawn(socket(self.input_1.pos));
		commands.spawn(socket(self.input_2.pos));
		commands.spawn(socket(self.output.pos));

		commands.spawn((
			NodeId(index),
			PickableBundle::default(),
			ShapeBundle {
				path: rounded_rect(self.center, self.width, self.height, 0.5),
				transform: Z_TRANSFORM_NODE,
				..default()
			},
			Fill {
				options: FillOptions::tolerance(0.05),
				color: color::NODE,
			},
			Stroke {
				options: StrokeOptions::default().with_line_width(3.0),
				color: color::NODE_SOCKET,
			},
		));
	}

	fn render(&self, _path: &mut Path) {}
}

fn rounded_rect(center: Vec2, width: f32, height: f32, radius: f32) -> Path {
	let half_width = width / 2.0;
	let half_height = height / 2.0;

	GeometryBuilder::build_as(&shapes::RoundedPolygon {
		points: vec![
			vec2(center.x - half_width, center.y + half_height), // top left
			vec2(center.x + half_width, center.y + half_height), // top right
			vec2(center.x + half_width, center.y - half_height), // bottom right
			vec2(center.x - half_width, center.y - half_height), // bottom left
		],
		radius,
		closed: true,
	})
}
