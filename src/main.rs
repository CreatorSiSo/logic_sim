use glam::{dvec2, vec2, DVec2};
#[allow(unused_imports)]
use log::{debug, info};
use petgraph::prelude::*;

use leptos::*;

// mod color;
mod nodes;
use nodes::{BinaryNode, InputNode, NodeVariant};
use web_sys::HtmlDivElement;

pub type Graph = petgraph::Graph<NodeVariant, (), Directed>;
pub type NodeIndex = petgraph::graph::NodeIndex;
pub type EdgeIndex = petgraph::graph::EdgeIndex;

fn main() {
	_ = console_log::init_with_level(log::Level::Debug);
	console_error_panic_hook::set_once();

	mount_to_body(|cx| {
		view! { cx, <Viewport/> }
	})
}

/// - minus sign to invert zoom direction (scroll forward => zoom in)
/// - 0.1 to slow down zooming
const ZOOM_SPEED: f64 = -0.1;

/// Using inverse of log2 for mapping zoom to scale
///
/// ## Zoom
/// - linear, -inf..+inf
/// - zoom(scale) = log2(scale)
///
/// ## Scale
/// - logarithmic, 0..+inf
/// - scale(zoom) = 2^(zoom)
///
/// ## Examples
/// ```txt
/// zoom => scale
///  4.0 => 16.0
///  3.0 => 8.0
///  2.0 => 4.0
///  1.0 => 2.0
///  0.0 => 1.0
/// -1.0 => 0.5
/// -2.0 => 0.25
/// ```
fn zoom_to_scale(zoom: f64) -> f64 {
	f64::powf(2.0, zoom)
}

#[component]
pub fn Viewport(cx: Scope) -> impl IntoView {
	let zoom = create_rw_signal(cx, 0.0);
	let origin = create_rw_signal(cx, dvec2(800.0, 340.0));
	let dragging = create_rw_signal(cx, false);

	// let target = event_target::<HtmlDivElement>(&wheel_event);

	let class = move || {
		"w-full h-full relative overflow-hidden select-none text-white bg-gray-900 ".to_string()
			+ if dragging() {
				"cursor-grabbing"
			} else {
				"cursor-grab"
			}
	};

	view! { cx,
		<div
			class=class
			on:wheel=move |wheel_event| {
				// zoom in or out
				let zoom_dir = if wheel_event.delta_y() > 0.0 { 1.0 } else { -1.0 };

				// relative to the origin, not the viewport
				let mouse_pos = dvec2(wheel_event.offset_x() as f64, wheel_event.offset_y() as f64) - origin();

				let old_scale = zoom_to_scale(zoom());
				zoom.update(|zoom| *zoom += zoom_dir * ZOOM_SPEED);
				let scale = zoom_to_scale(zoom());

				// movement of the point at the mouse position due to scaling
				let diff = mouse_pos / scale - mouse_pos / old_scale;
				origin.update(|origin| *origin += diff * scale);

			}
			on:pointermove=move |pointer_event| {
				if dragging() {
					origin.update(|origin| {
						origin.x += pointer_event.movement_x() as f64;
						origin.y += pointer_event.movement_y() as f64;
					})
				}
			}
			on:pointerdown=move |_| dragging.set(true)
			on:pointerup=move |_| dragging.set(false)
			// TODO Continue panning if mouse leaves viewport
			on:pointerleave=move |_| dragging.set(false)
		>
			<div
				class="absolute /top-1/2 /left-1/2"
				style=move || {
					let scale = zoom_to_scale(zoom());
					format!(
						"transform: scale({}) translate({}px, {}px)",
						scale, origin().x / scale, origin().y / scale
					)
				}
			>
				<Node
					inputs=&[true, true]
					outputs=&[false]
					position=dvec2(-200.0, -200.0)
				/>
				<Node
					inputs=&[false, true]
					outputs=&[true]
					position=dvec2(0.0, 0.0)
				/>
				<Node
					inputs=&[true, false]
					outputs=&[true]
					position=dvec2(200.0, 200.0)
				/>
			</div>
		</div>
	}
}

#[component]
pub fn Node<'a>(
	cx: Scope,
	inputs: &'a [bool],
	outputs: &'a [bool],
	position: DVec2,
) -> impl IntoView {
	let (inputs, _) = create_signal(
		cx,
		inputs
			.iter()
			.map(|input| create_rw_signal(cx, *input))
			.collect::<Vec<_>>(),
	);

	view! { cx,
		<span
			class="absolute flex flex-col gap-1 py-1 rounded cursor-pointer bg-gray-600 border-[3px] border-gray-800 drop-shadow-lg"
			style=format!("transform: translate({}px, {}px)", position.x, position.y)
		>
			{
				inputs()
					.iter()
					.map(|input| view! { cx, <NodeSection state=*input /> })
					.collect::<Vec<_>>()
			}
		</span>
	}
}

enum Align {
	Left,
	Right,
}

#[component]
fn NodeSection(cx: Scope, state: RwSignal<bool>) -> impl IntoView {
	let socket_class = move || {
		"absolute w-[14px] h-[14px] rounded-full border-2 border-gray-800 ".to_string()
			+ if state() { "bg-red-400" } else { "bg-gray-300" }
	};
	view! { cx,
		<div class="relative">
			<span
				on:click=move |_| state.update(|state| *state = !*state)
				class=socket_class
				style="top: 50%; transform: translate(-7px, -7px);"
			>
			</span>
			<div class="px-3">
				"Label"
			</div>
		</div>
	}
}

const MAX_Z: f32 = 100.0;

fn setup() -> Graph {
	let mut graph = Graph::default();
	let in_1 = graph.add_node(InputNode::new(false, vec2(0.0, 0.0)).into());
	let in_2 = graph.add_node(InputNode::new(false, vec2(0.0, 10.)).into());
	let in_3 = graph.add_node(InputNode::new(false, vec2(0.0, 20.)).into());
	let in_4 = graph.add_node(InputNode::new(false, vec2(0.0, 30.)).into());
	graph.add_node(BinaryNode::new(vec2(300., 0.), 200., 80.).into());

	let node_2 = graph.add_node(NodeVariant::Void);

	graph.add_edge(in_1, node_2, ());
	graph.add_edge(in_2, node_2, ());
	graph.add_edge(in_3, node_2, ());
	graph.add_edge(in_4, node_2, ());

	graph
}

fn update_graph(mut graph: &mut Graph) {
	for weight in graph.node_weights_mut() {
		match weight {
			NodeVariant::In(InputNode { .. }) => {
				// *pos += Vec2::new(0.1, 0.0);
			}
			_ => {}
		}
	}
}

// fn interactions(
// 	mut events: EventReader<PickingEvent>,
// 	mut graph: Query<&mut GraphWrapper>,
// 	mut nodes: Query<(&mut NodeId, &mut Fill)>,
// ) {
// 	let graph = &mut graph.single_mut().0;

// 	fn get_data_mut<'a>(
// 		graph: &'a mut Graph,
// 		nodes: &'a mut Query<(&mut NodeId, &mut Fill)>,
// 		entity: &Entity,
// 	) -> (&'a mut NodeVariant, Mut<'a, Fill>) {
// 		let (node_id, fill) = nodes.get_mut(*entity).unwrap();
// 		(graph.node_weight_mut(node_id.0).unwrap(), fill)
// 	}

// 	let Some(event) = events.iter().next() else {
// 		return;
// 	};

// 	// TODO figure out why events are sometimes received twice

// 	match event {
// 		PickingEvent::Hover(hover_event) => match hover_event {
// 			HoverEvent::JustEntered(entity) => {
// 				let (node, mut fill) = get_data_mut(graph, &mut nodes, entity);
// 				if let NodeVariant::In(InputNode { state, .. }) = node {
// 					if !*state {
// 						fill.color = color::NODE_SOCKET_HOVERED;
// 					}
// 				}
// 			}
// 			HoverEvent::JustLeft(entity) => {
// 				let (node, mut fill) = get_data_mut(graph, &mut nodes, entity);
// 				if let NodeVariant::In(InputNode { state, .. }) = node {
// 					if !*state {
// 						fill.color = color::NODE_SOCKET;
// 					}
// 				}
// 			}
// 		},
// 		PickingEvent::Clicked(entity) => {
// 			let (node, mut fill) = get_data_mut(graph, &mut nodes, entity);
// 			if let NodeVariant::In(InputNode { state, .. }) = node {
// 				*state = !*state;
// 				fill.color = if *state {
// 					color::ACTIVE
// 				} else {
// 					color::NODE_SOCKET
// 				};
// 			}
// 		}
// 		_ => {}
// 	}
// }

// fn render_edges(
// 	mut commands: Commands,
// 	world_cursor: Res<WorldCursor>,
// 	// mouse_buttons: Res<Input<MouseButton>>,
// 	mut graph: Query<&mut GraphWrapper>,
// 	mut query: Query<(&mut EdgePart, &mut Path, &mut Stroke)>,
// ) {
// 	let graph = &mut graph.single_mut().0;

// 	let start_indices: Vec<NodeIndex> = graph
// 		.node_indices()
// 		.zip(graph.node_weights())
// 		.filter_map(|(index, node)| matches!(node, NodeVariant::In(_)).then_some(index))
// 		.collect();

// 	for start_index in start_indices {
// 		let mut visitor = petgraph::visit::Dfs::new(&*graph, start_index);
// 		while let Some(index) = visitor.next(&*graph) {
// 			for edge in graph.edges(index) {
// 				let index = edge.id();
// 				let source = graph.node_weight(edge.source()).unwrap();
// 				// let target = graph.node_weight(edge.target()).unwrap();

// 				let source_pos = match source {
// 					NodeVariant::In(input_node) => input_node.pos,
// 					_ => panic!(),
// 				};
// 				// let target_pos = match target {
// 				// 	LogicNode::In(input_node) => input_node.pos,
// 				// 	LogicNode::Void => Vec2::new(10.0, 10.0),
// 				// };
// 				let mouse_pos = world_cursor.pos.unwrap_or(vec2(0.0, 0.0));

// 				if let Some((_, mut path, _)) = query
// 					.iter_mut()
// 					.find(|(edge_part, ..)| edge_part.index == index)
// 				{
// 					*path = edge_path(source_pos, mouse_pos);
// 				} else {
// 					commands.spawn((
// 						EdgePart { index },
// 						ShapeBundle {
// 							path: edge_path(source_pos, mouse_pos),
// 							transform: Transform::from_translation(vec3(0.0, 0.0, 1.0)),
// 							..default()
// 						},
// 						Stroke {
// 							options: StrokeOptions::DEFAULT
// 								.with_line_cap(LineCap::Round)
// 								.with_line_width(2.5),
// 							color: color::EDGE,
// 						},
// 					));
// 					continue;
// 				};
// 			}
// 		}
// 	}
// }

// fn edge_path(start_pos: Vec2, end_pos: Vec2) -> Path {
// 	let mut path_builder = PathBuilder::new();
// 	path_builder.move_to(start_pos);
// 	path_builder.line_to(end_pos);
// 	path_builder.build()
// }
