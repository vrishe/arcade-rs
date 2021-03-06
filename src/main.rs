#![allow(dead_code)]

extern crate sdl2;

#[macro_use]
mod macros;

mod hud;
mod phi;
mod views;

#[cfg(feature="debug")]
const DEBUG: bool = true;
#[cfg(not(feature="debug"))]
const DEBUG: bool= false;

fn main() {
	::phi::spawn("ArcadeRS", (800, 600), |phi| {
		Box::new(::views::menu_main::MainMenuView::new(phi))
	});
}
