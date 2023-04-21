use bevy::{
	core_pipeline::clear_color::ClearColorConfig,
	math::{vec2, vec3},
	prelude::*,
};
use bevy_mod_picking::{HoverEvent, PickingCameraBundle, PickingEvent};
use bevy_prototype_lyon::prelude::*;
use petgraph::prelude::*;

mod simulation;
use simulation::{EdgeIndex, Graph, InputNode, Node, NodeIndex, NodeType};

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
struct NodeWrapper {
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
	{
		let mut graph = Graph::default();
		let in_1 = graph.add_node(InputNode::new(false, Vec2::new(0.0, 3.2 + 0.4)).into());
		let in_2 = graph.add_node(InputNode::new(false, Vec2::new(0.0, 1.0 + 0.2)).into());
		let in_3 = graph.add_node(InputNode::new(false, Vec2::new(0.0, -1.0 - 0.2)).into());
		let in_4 = graph.add_node(InputNode::new(false, Vec2::new(0.0, -3.2 - 0.4)).into());

		let node_2 = graph.add_node(NodeType::Void);

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
				clear_color: ClearColorConfig::Custom(Color::rgb(0.1, 0.1, 0.1)),
			},
			transform: Transform::from_scale(Vec3::new(0.1, 0.1, 1.0)),
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
			NodeType::In(InputNode { .. }) => {
				// *pos += Vec2::new(0.1, 0.0);
			}
			_ => {}
		}
	}
}

fn interactions(
	mut events: EventReader<PickingEvent>,
	mut graph: Query<&mut GraphWrapper>,
	mut nodes: Query<(&mut NodeWrapper, &mut Fill)>,
) {
	let graph = &mut graph.single_mut().0;

	fn get_data_mut<'a>(
		graph: &'a mut Graph,
		nodes: &'a mut Query<(&mut NodeWrapper, &mut Fill)>,
		entity: &Entity,
	) -> (&'a mut NodeType, Mut<'a, Fill>) {
		let (node_link, fill) = nodes.get_mut(*entity).unwrap();
		(graph.node_weight_mut(node_link.index).unwrap(), fill)
	}

	let Some(event) = events.iter().next() else {
		return;
	};

	// TODO figure out why events are sometimes received twice

	match event {
		PickingEvent::Hover(hover_event) => match hover_event {
			HoverEvent::JustEntered(entity) => {
				let (node, mut fill) = get_data_mut(graph, &mut nodes, entity);
				if let NodeType::In(InputNode { state, .. }) = node {
					if !*state {
						fill.color = COLOR_NODE_BG_HOVERED;
					}
				}
			}
			HoverEvent::JustLeft(entity) => {
				let (node, mut fill) = get_data_mut(graph, &mut nodes, entity);
				if let NodeType::In(InputNode { state, .. }) = node {
					if !*state {
						fill.color = COLOR_NODE_BG;
					}
				}
			}
		},
		PickingEvent::Clicked(entity) => {
			let (node, mut fill) = get_data_mut(graph, &mut nodes, entity);
			if let NodeType::In(InputNode { state, .. }) = node {
				*state = !*state;
				fill.color = if *state { COLOR_ACTIVE } else { COLOR_NODE_BG };
			}
		}
		_ => {}
	}
}

fn render_nodes(
	mut commands: Commands,
	mut graph: Query<&mut GraphWrapper>,
	mut nodes: Query<(&mut NodeWrapper, &mut Path)>,
) {
	let graph = &mut graph.single_mut().0;

	let indices: Vec<NodeIndex> = graph.node_indices().collect();
	for (weight, index) in graph.node_weights_mut().zip(indices) {
		if let Some((_, mut path)) = nodes
			.iter_mut()
			.find(|(node_link, ..)| node_link.index == index)
		{
			weight.render(&mut path);
		} else {
			weight.init(&mut commands, index);
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
		.filter_map(|(index, node)| matches!(node, NodeType::In(_)).then_some(index))
		.collect();

	for start_index in start_indices {
		let mut visitor = petgraph::visit::Dfs::new(&*graph, start_index);
		while let Some(index) = visitor.next(&*graph) {
			for edge in graph.edges(index) {
				let index = edge.id();
				let source = graph.node_weight(edge.source()).unwrap();
				// let target = graph.node_weight(edge.target()).unwrap();

				let source_pos = match source {
					NodeType::In(input_node) => input_node.pos,
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
