use bevy::prelude::*;
use petgraph::Directed;

struct Edge {
	input: Node,
	output: Node,
}

impl Edge {
	fn new() -> Self {
		Self {
			input: Node::Void,
			output: Node::Void,
		}
	}
}

enum Node {
	In(InputNode),
	Void,
}

struct InputNode {
	state: bool,
}

impl InputNode {
	fn new(state: bool) -> Self {
		Self { state }
	}
}

#[derive(Component)]
struct Graph(petgraph::Graph<Node, (), Directed>);

impl Graph {
	fn new() -> Self {
		Self(petgraph::Graph::default())
	}
}

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_startup_system(setup_graph)
		.add_system(update_graph)
		.run();
}

fn setup_graph(mut commands: Commands) {
	// let mut graph: Graph<Node, (), Directed> = petgraph::Graph::default();
	// let node_1 = graph.add_node(Node::In(InputNode::new(false)));
	// let node_2 = graph.add_node(Node::Void);
	// let edge_1 = graph.add_edge(node_1, node_2, ());

	// graph.neighbors(node_1);

	commands.spawn(Graph::new());
}

fn update_graph(mut query: Query<&mut Graph>) {
	query.par_iter_mut().for_each_mut(|mut graph| {
		println!("Updating Graph ...");
		graph.0.clear();
	});
}
