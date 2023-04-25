use bevy::{prelude::*, ui::FocusPolicy};
use bevy_mod_picking::PickableBundle;
use bevy_prototype_lyon::prelude::*;

use crate::{color, NodeId};

use super::{z_transform, UiElement};

#[derive(Debug, Default)]
pub struct NodeSocket<S> {
	index: u32,
	// Position relative to the nodes origin
	pos: Vec2,
	state: Option<S>,
}

impl<S> NodeSocket<S> {
	pub fn new(index: u32, pos: Vec2, state: Option<S>) -> Self {
		Self { index, pos, state }
	}
}

impl<S> UiElement for NodeSocket<S> {
	fn init(&self, commands: &mut Commands, node_index: crate::NodeIndex) {
		commands.spawn((
			NodeId(node_index),
			PickableBundle {
				focus_policy: FocusPolicy::Block,
				..default()
			},
			ShapeBundle {
				path: GeometryBuilder::build_as(&shapes::Circle {
					radius: 5.0,
					center: self.pos,
				}),
				transform: z_transform(node_index, 0.1),
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
		));
	}

	fn render(&self, path: &mut Path) {
		todo!()
	}
}
