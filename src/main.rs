use bevy::{
	core_pipeline::clear_color::ClearColorConfig,
	math::{vec2, vec3},
	prelude::*,
};
use bevy_mod_picking::{HoverEvent, PickingCameraBundle, PickingEvent};
use bevy_prototype_lyon::prelude::*;
use petgraph::prelude::*;

mod color;
mod nodes;
use nodes::{BinaryNode, InputNode, NodeVariant, UiElement};

pub type Graph = petgraph::Graph<NodeVariant, (), Directed>;
pub type NodeIndex = petgraph::graph::NodeIndex;
pub type EdgeIndex = petgraph::graph::EdgeIndex;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugin(bevy_prototype_lyon::plugin::ShapePlugin)
		.add_plugins(bevy_mod_picking::DefaultPickingPlugins)
		.add_plugin(bevy_mod_picking::DebugEventsPickingPlugin)
		// Startup
		.add_startup_system(setup)
		// Update
		.add_systems((update_cursor, update_graph, interactions))
		// Render
		.add_systems((render_nodes, render_edges))
		.run();
}

#[derive(Component, Debug)]
struct GraphWrapper(Graph);

#[derive(Component, Debug)]
struct NodeId(NodeIndex);
#[derive(Component, Debug)]
struct EdgePart {
	index: EdgeIndex,
}

const MAX_Z: f32 = 100.0;

#[derive(Debug, Component)]
struct MainCamera;

#[derive(Debug, Resource)]
struct WorldCursor {
	pos: Option<Vec2>,
}

fn setup(mut commands: Commands) {
	{
		let mut graph = Graph::default();
		let in_1 = graph.add_node(InputNode::new(false, Vec2::new(0.0, 0.0)).into());
		let in_2 = graph.add_node(InputNode::new(false, Vec2::new(0.0, 10.)).into());
		let in_3 = graph.add_node(InputNode::new(false, Vec2::new(0.0, 20.)).into());
		let in_4 = graph.add_node(InputNode::new(false, Vec2::new(0.0, 30.)).into());
		graph.add_node(BinaryNode::new(Vec2::new(300., 0.), 200., 80.).into());

		let node_2 = graph.add_node(NodeVariant::Void);

		graph.add_edge(in_1, node_2, ());
		graph.add_edge(in_2, node_2, ());
		graph.add_edge(in_3, node_2, ());
		graph.add_edge(in_4, node_2, ());

		commands.spawn(GraphWrapper(graph));
	}

	commands.insert_resource(WorldCursor { pos: None });
	commands.spawn((
		MainCamera,
		PickingCameraBundle { ..default() },
		Camera2dBundle {
			camera_2d: Camera2d {
				clear_color: ClearColorConfig::Custom(color::BG),
			},
			transform: Transform {
				translation: vec3(0.0, 0.0, MAX_Z),
				scale: Vec3::splat(1.0),
				..default()
			},
			..default()
		},
	));
}

fn update_cursor(
	camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
	mut cursor_moved: EventReader<CursorMoved>,
	mut world_cursor: ResMut<WorldCursor>,
) {
	let (camera, camera_transform) = camera_query.single();
	let Some(cursor_moved) = cursor_moved.iter().next() else {
		return;
	};
	world_cursor.pos = camera
		.viewport_to_world(camera_transform, cursor_moved.position)
		.map(|ray| ray.origin.truncate());
}

fn update_graph(mut graph: Query<&mut GraphWrapper>) {
	let graph = &mut graph.single_mut().0;

	for weight in graph.node_weights_mut() {
		match weight {
			NodeVariant::In(InputNode { .. }) => {
				// *pos += Vec2::new(0.1, 0.0);
			}
			_ => {}
		}
	}
}

fn interactions(
	mut events: EventReader<PickingEvent>,
	mut graph: Query<&mut GraphWrapper>,
	mut nodes: Query<(&mut NodeId, &mut Fill)>,
) {
	let graph = &mut graph.single_mut().0;

	fn get_data_mut<'a>(
		graph: &'a mut Graph,
		nodes: &'a mut Query<(&mut NodeId, &mut Fill)>,
		entity: &Entity,
	) -> (&'a mut NodeVariant, Mut<'a, Fill>) {
		let (node_id, fill) = nodes.get_mut(*entity).unwrap();
		(graph.node_weight_mut(node_id.0).unwrap(), fill)
	}

	let Some(event) = events.iter().next() else {
		return;
	};

	// TODO figure out why events are sometimes received twice

	match event {
		PickingEvent::Hover(hover_event) => match hover_event {
			HoverEvent::JustEntered(entity) => {
				let (node, mut fill) = get_data_mut(graph, &mut nodes, entity);
				if let NodeVariant::In(InputNode { state, .. }) = node {
					if !*state {
						fill.color = color::NODE_SOCKET_HOVERED;
					}
				}
			}
			HoverEvent::JustLeft(entity) => {
				let (node, mut fill) = get_data_mut(graph, &mut nodes, entity);
				if let NodeVariant::In(InputNode { state, .. }) = node {
					if !*state {
						fill.color = color::NODE_SOCKET;
					}
				}
			}
		},
		PickingEvent::Clicked(entity) => {
			let (node, mut fill) = get_data_mut(graph, &mut nodes, entity);
			if let NodeVariant::In(InputNode { state, .. }) = node {
				*state = !*state;
				fill.color = if *state {
					color::ACTIVE
				} else {
					color::NODE_SOCKET
				};
			}
		}
		_ => {}
	}
}

fn render_nodes(
	mut commands: Commands,
	mut graph: Query<&mut GraphWrapper>,
	mut nodes: Query<(&mut NodeId, &mut Path)>,
) {
	let graph = &mut graph.single_mut().0;
	let indices = graph.node_indices();
	let data_indices = graph.node_weights_mut().zip(indices);

	for (data, index) in data_indices {
		if let Some((_, mut path)) = nodes.iter_mut().find(|(node_id, ..)| node_id.0 == index) {
			data.render(&mut path);
		} else {
			data.init(&mut commands, index);
		}
	}
}

fn render_edges(
	mut commands: Commands,
	world_cursor: Res<WorldCursor>,
	// mouse_buttons: Res<Input<MouseButton>>,
	mut graph: Query<&mut GraphWrapper>,
	mut query: Query<(&mut EdgePart, &mut Path, &mut Stroke)>,
) {
	let graph = &mut graph.single_mut().0;

	let start_indices: Vec<NodeIndex> = graph
		.node_indices()
		.zip(graph.node_weights())
		.filter_map(|(index, node)| matches!(node, NodeVariant::In(_)).then_some(index))
		.collect();

	for start_index in start_indices {
		let mut visitor = petgraph::visit::Dfs::new(&*graph, start_index);
		while let Some(index) = visitor.next(&*graph) {
			for edge in graph.edges(index) {
				let index = edge.id();
				let source = graph.node_weight(edge.source()).unwrap();
				// let target = graph.node_weight(edge.target()).unwrap();

				let source_pos = match source {
					NodeVariant::In(input_node) => input_node.pos,
					_ => panic!(),
				};
				// let target_pos = match target {
				// 	LogicNode::In(input_node) => input_node.pos,
				// 	LogicNode::Void => Vec2::new(10.0, 10.0),
				// };
				let mouse_pos = world_cursor.pos.unwrap_or(vec2(0.0, 0.0));

				if let Some((_, mut path, _)) = query
					.iter_mut()
					.find(|(edge_part, ..)| edge_part.index == index)
				{
					*path = edge_path(source_pos, mouse_pos);
				} else {
					commands.spawn((
						EdgePart { index },
						ShapeBundle {
							path: edge_path(source_pos, mouse_pos),
							transform: Transform::from_translation(vec3(0.0, 0.0, 1.0)),
							..default()
						},
						Stroke {
							options: StrokeOptions::DEFAULT
								.with_line_cap(LineCap::Round)
								.with_line_width(2.5),
							color: color::EDGE,
						},
					));
					continue;
				};
			}
		}
	}
}

fn edge_path(start_pos: Vec2, end_pos: Vec2) -> Path {
	let mut path_builder = PathBuilder::new();
	path_builder.move_to(start_pos);
	path_builder.line_to(end_pos);
	path_builder.build()
}
