use crate::{NodeId, NodeIndex};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

mod input_node;
pub use input_node::InputNode;

mod binary_node;
pub use binary_node::BinaryNode;

pub trait Node {
	fn init(&self, commands: &mut Commands, index: NodeIndex);
	fn render(&self, path: &mut Path);
}

#[derive(Debug)]
pub enum NodeVariant {
	In(InputNode),
	Binary(BinaryNode),
	Unary(UnaryNode),
	Void,
}

impl Node for NodeVariant {
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
