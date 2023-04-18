use bevy::{
	core_pipeline::clear_color::ClearColorConfig,
	math::Vec3Swizzles,
	prelude::{shape::Circle, *},
	sprite::MaterialMesh2dBundle,
};
use petgraph::{graph::NodeIndex, Directed};

type Graph = petgraph::Graph<LogicNode, (), Directed>;

#[derive(Debug)]
enum LogicNode {
	In(InputNode),
	Void,
}

#[derive(Debug)]
struct InputNode {
	state: bool,
	pos: Vec2,
}

impl InputNode {
	fn new(state: bool, pos: Vec2) -> Self {
		Self { state, pos }
	}
}

#[derive(Component, Debug)]
struct GraphWrapper(Graph);

#[derive(Component, Debug)]
struct NodeSocket {
	index: NodeIndex,
}

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_startup_system(setup)
		.add_system(update_cursor)
		.add_system(update_graph)
		.add_system(render_nodes)
		.run();
}

#[derive(Debug, Resource)]
struct MaterialHandles {
	node_bg: Handle<ColorMaterial>,
	node_bg_hovered: Handle<ColorMaterial>,
	node_bg_focused: Handle<ColorMaterial>,
	active: Handle<ColorMaterial>,
}

#[derive(Debug, Component)]
struct MainCamera;

#[derive(Debug, Resource)]
struct WorldCursor {
	pos: Option<Vec2>,
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
	let mut graph = Graph::default();
	let in_1 = graph.add_node(LogicNode::In(InputNode::new(false, Vec2::new(0.0, 90.0))));
	let in_2 = graph.add_node(LogicNode::In(InputNode::new(false, Vec2::new(0.0, 60.0))));
	let in_3 = graph.add_node(LogicNode::In(InputNode::new(false, Vec2::new(0.0, 30.0))));
	let in_4 = graph.add_node(LogicNode::In(InputNode::new(false, Vec2::new(0.0, 0.0))));

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
			..default()
		},
	));

	commands.insert_resource(MaterialHandles {
		node_bg: materials.add(ColorMaterial::from(Color::rgb(0.2, 0.2, 0.2))),
		node_bg_hovered: materials.add(ColorMaterial::from(Color::rgb(0.15, 0.15, 0.15))),
		node_bg_focused: materials.add(ColorMaterial::from(Color::rgb(0.3, 0.3, 0.3))),
		active: materials.add(ColorMaterial::from(Color::rgb(1.0, 0.1, 0.1))),
	});
	commands.insert_resource(WorldCursor { pos: None })
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
				*pos += Vec2::new(0.1, 0.0);
			}
			LogicNode::Void => {}
		}
	}
}

fn render_nodes(
	mut commands: Commands,
	world_cursor: Res<WorldCursor>,
	graph: Query<&GraphWrapper>,
	mut node_sockets: Query<(&mut NodeSocket, &mut Transform, &mut Handle<ColorMaterial>)>,
	mut meshes: ResMut<Assets<Mesh>>,
	materials: Res<MaterialHandles>,
) {
	let graph = &graph.single().0;
	// node_sockets
	// 	.iter()
	// 	.inspect(|n| println!("{n:?}"))
	// 	.for_each(drop);

	for (weight, index) in graph.node_weights().zip(graph.node_indices()) {
		match weight {
			LogicNode::In(InputNode { pos, .. }) => {
				let Some((_, mut transform, mut material)) = node_sockets
					.iter_mut()
					.find(|(node_socket, ..)| node_socket.index == index)
				else {
					commands.spawn((
						NodeSocket { index },
						MaterialMesh2dBundle {
							mesh: meshes.add(Circle::new(10.).into()).into(),
							material: materials.node_bg.clone(),
							transform: Transform::from_translation(Vec3::new(pos.x, pos.y, 0.0)),
							..default()
						},
					));
					continue;
				};

				transform.translation.x = pos.x;
				transform.translation.y = pos.y;

				dbg!(&world_cursor);
				if world_cursor
					.pos
					.map_or(false, |pos| pos.distance(transform.translation.xy()) < 10.0)
				{
					*material = materials.node_bg_hovered.clone();
					println!("updated");
				} else {
					*material = materials.node_bg.clone();
				}
			}
			LogicNode::Void => {}
		}
	}
}
