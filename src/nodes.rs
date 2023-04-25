use crate::{NodeId, NodeIndex};
use bevy::{math::vec2, prelude::*};
use bevy_prototype_lyon::prelude::*;

mod node_socket;
pub use node_socket::NodeSocket;

mod input_node;
pub use input_node::InputNode;

mod binary_node;
pub use binary_node::BinaryNode;

pub trait UiElement {
	fn init(&self, commands: &mut Commands, node_index: NodeIndex);
	fn render(&self, path: &mut Path);
}

#[derive(Debug)]
pub enum NodeVariant {
	In(InputNode),
	Binary(BinaryNode),
	Unary(UnaryNode),
	Void,
}

impl UiElement for NodeVariant {
	fn init(&self, commands: &mut Commands, index: NodeIndex) {
		match self {
			NodeVariant::In(input_node) => input_node.init(commands, index),
			NodeVariant::Binary(binary_node) => binary_node.init(commands, index),
			NodeVariant::Void => {
				commands.spawn(NodeId(index));
			}
			_ => info!("Not initializing {self:?}"),
		}
	}

	fn render(&self, path: &mut Path) {
		match self {
			NodeVariant::In(input_node) => input_node.render(path),
			NodeVariant::Binary(binary_node) => binary_node.render(path),
			// NodeVariant::Unary(unary_node) => unary_node.render(path),
			_ => info!("Not rendering {self:?}"),
		}
	}
}

impl From<InputNode> for NodeVariant {
	fn from(other: InputNode) -> Self {
		Self::In(other)
	}
}

impl From<BinaryNode> for NodeVariant {
	fn from(other: BinaryNode) -> Self {
		Self::Binary(other)
	}
}

#[derive(Debug)]
pub struct UnaryNode {}

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

// TODO introduce proper sorting
pub fn z_transform(node_index: NodeIndex, offset: f32) -> Transform {
	Transform::from_xyz(
		0.0,
		0.0,
		node_index.index() as f32 / u32::MAX as f32 + offset,
	)
}
