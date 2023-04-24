use bevy::prelude::Color;

macro_rules! color_rgb_u8 {
	($rgb:expr) => {
		Color::rgb(
			$rgb as f32 / u8::MAX as f32,
			$rgb as f32 / u8::MAX as f32,
			$rgb as f32 / u8::MAX as f32,
		)
	};
	($r:expr, $g:expr, $b:expr) => {
		Color::rgb(
			$r as f32 / u8::MAX as f32,
			$g as f32 / u8::MAX as f32,
			$b as f32 / u8::MAX as f32,
		)
	};
}

pub const BG: Color = color_rgb_u8!(22);

pub const NODE: Color = color_rgb_u8!(48);
pub const NODE_HOVERED: Color = color_rgb_u8!(48);

pub const NODE_SOCKET: Color = color_rgb_u8!(34);
pub const NODE_SOCKET_HOVERED: Color = color_rgb_u8!(40);

pub const ACTIVE: Color = Color::rgb(1.0, 0.1, 0.1);
pub const EDGE: Color = Color::rgb(0.4, 0.4, 0.4);
