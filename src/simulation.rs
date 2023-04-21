use bevy::{prelude::*, ui::FocusPolicy};
use bevy_mod_picking::{NoDeselect, PickableBundle};
use bevy_prototype_lyon::{prelude::*, shapes};
use petgraph::Directed;

use crate::{NodeWrapper, COLOR_NODE_BG};

pub type Graph = petgraph::Graph<NodeType, (), Directed>;
pub type NodeIndex = petgraph::graph::NodeIndex;
pub type EdgeIndex = petgraph::graph::EdgeIndex;

pub trait Node {
	fn init(&self, commands: &mut Commands, index: NodeIndex);
	fn render(&self, path: &mut Path);
}

#[derive(Debug)]
pub enum NodeType {
	In(InputNode),
	Binary(BinaryNode),
	Unary(UnaryNode),
	Void,
}

impl Node for NodeType {
	fn init(&self, commands: &mut Commands, index: NodeIndex) {
		match self {
			NodeType::In(in_node) => in_node.init(commands, index),
			NodeType::Binary(_) | NodeType::Unary(_) | NodeType::Void => {}
		}
	}

	fn render(&self, path: &mut Path) {}
}

impl From<InputNode> for NodeType {
	fn from(input_node: InputNode) -> Self {
		Self::In(input_node)
	}
}

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
			PickableBundle {
				focus_policy: FocusPolicy::Block,
				..default()
			},
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

	fn render(&self, path: &mut Path) {
		*path = GeometryBuilder::build_as(&shapes::Circle {
			radius: 1.0,
			center: self.pos,
		});
	}
}

#[derive(Debug)]
pub struct BinaryNode {}

#[derive(Debug)]
pub struct UnaryNode {}
