use bevy::{
	core_pipeline::clear_color::ClearColorConfig,
	math::{vec2, vec3},
	prelude::*,
};
use bevy_prototype_lyon::prelude::*;
use petgraph::prelude::*;

mod sim {
	use bevy::prelude::Vec2;
	use petgraph::Directed;

	pub type Graph = petgraph::Graph<LogicNode, (), Directed>;
	pub type NodeIndex = petgraph::graph::NodeIndex;
	pub type EdgeIndex = petgraph::graph::EdgeIndex;

	#[derive(Debug)]
	pub enum LogicNode {
		In(InputNode),
		Void,
	}

	impl From<InputNode> for LogicNode {
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
}

use sim::{EdgeIndex, Graph, InputNode, LogicNode, NodeIndex};

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugin(ShapePlugin)
		.add_startup_system(setup)
		.add_system(update_cursor)
		.add_system(update_graph)
		.add_system(render_nodes)
		.add_system(render_edges)
		.run();
}

#[derive(Component, Debug)]
struct GraphWrapper(Graph);

#[derive(Component, Debug)]
struct NodePart {
	index: NodeIndex,
}

#[derive(Component, Debug)]
struct EdgePart {
	index: EdgeIndex,
}

const COLOR_NODE_BG: Color = Color::rgb(0.2, 0.2, 0.2);
const COLOR_NODE_BG_HOVERED: Color = Color::rgb(0.15, 0.15, 0.15);
const COLOR_NODE_BG_FOCUSED: Color = Color::rgb(0.3, 0.3, 0.3);
const COLOR_ACTIVE: Color = Color::rgb(1.0, 0.1, 0.1);

#[derive(Debug, Component)]
struct MainCamera;

#[derive(Debug, Resource)]
struct WorldCursor {
	pos: Option<Vec2>,
}

fn setup(mut commands: Commands) {
	let mut graph = Graph::default();
	let in_1 = graph.add_node(InputNode::new(false, Vec2::new(0.0, 3.2 + 0.4)).into());
	let in_2 = graph.add_node(InputNode::new(false, Vec2::new(0.0, 1.0 + 0.2)).into());
	let in_3 = graph.add_node(InputNode::new(false, Vec2::new(0.0, -1.0 - 0.2)).into());
	let in_4 = graph.add_node(InputNode::new(false, Vec2::new(0.0, -3.2 - 0.4)).into());

	let node_2 = graph.add_node(LogicNode::Void);

	graph.add_edge(in_1, node_2, ());
	graph.add_edge(in_2, node_2, ());
	graph.add_edge(in_3, node_2, ());
	graph.add_edge(in_4, node_2, ());

	commands.spawn(GraphWrapper(graph));
	commands.spawn((
		MainCamera,
		Camera2dBundle {
			camera_2d: Camera2d {
				clear_color: ClearColorConfig::Custom(Color::rgb(0.1, 0.1, 0.1)),
			},
			transform: Transform::from_scale(Vec3::new(0.1, 0.1, 1.0)),
			..default()
		},
	));

	commands.insert_resource(WorldCursor { pos: None });
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
			LogicNode::In(InputNode { pos, .. }) => {
				// *pos += Vec2::new(0.1, 0.0);
			}
			LogicNode::Void => {}
		}
	}
}

fn render_nodes(
	mut commands: Commands,
	world_cursor: Res<WorldCursor>,
	mouse_buttons: Res<Input<MouseButton>>,
	mut graph: Query<&mut GraphWrapper>,
	mut node_sockets: Query<(&mut NodePart, &mut Path, &mut Fill)>,
) {
	let graph = &mut graph.single_mut().0;

	let indices: Vec<NodeIndex> = graph.node_indices().collect();
	for (weight, index) in graph.node_weights_mut().zip(indices) {
		match weight {
			LogicNode::In(InputNode {
				pos: node_pos,
				state,
			}) => {
				let Some((_, mut path, mut fill)) = node_sockets
					.iter_mut()
					.find(|(node_socket, ..)| node_socket.index == index)
				else {
					commands.spawn((
						NodePart { index },
						ShapeBundle {
							path: GeometryBuilder::build_as(&shapes::Circle { radius: 1.0, center: *node_pos }),
							..default()
						},
						Fill {
							options: FillOptions::tolerance(0.05),
							color: COLOR_NODE_BG
						}
					));
					continue;
				};

				*path = GeometryBuilder::build_as(&shapes::Circle {
					radius: 1.0,
					center: *node_pos,
				});

				let hovered = world_cursor
					.pos
					.map_or(false, |cursor_pos| cursor_pos.distance(*node_pos) < 1.0);

				if hovered && mouse_buttons.just_released(MouseButton::Left) {
					*state = !*state;
				}

				fill.color = match (*state, hovered) {
					(true, true) | (true, false) => COLOR_ACTIVE,
					(false, true) => COLOR_NODE_BG_HOVERED,
					(false, false) => COLOR_NODE_BG,
				};
			}
			LogicNode::Void => {}
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
		.filter_map(|(index, node)| matches!(node, LogicNode::In(_)).then_some(index))
		.collect();

	for start_index in start_indices {
		let mut visitor = petgraph::visit::Dfs::new(&*graph, start_index);
		while let Some(index) = visitor.next(&*graph) {
			for edge in graph.edges(index) {
				let index = edge.id();
				let source = graph.node_weight(edge.source()).unwrap();
				// let target = graph.node_weight(edge.target()).unwrap();

				let source_pos = match source {
					LogicNode::In(input_node) => input_node.pos,
					LogicNode::Void => panic!(),
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
							transform: Transform::from_translation(vec3(0.0, 0.0, -1.0)),
							..default()
						},
						Stroke {
							options: StrokeOptions::DEFAULT
								.with_line_cap(LineCap::Round)
								.with_line_width(0.5),
							color: COLOR_NODE_BG_FOCUSED,
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
