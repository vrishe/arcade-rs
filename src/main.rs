#![allow(dead_code)]

extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_mixer;
extern crate sdl2_ttf;


mod phi;
mod views;

#[cfg(feature="debug")]
const DEBUG: bool = true;
#[cfg(not(feature="debug"))]
const DEBUG: bool= false;


fn main() {
	::phi::spawn("ArcadeRS Shooter", (800, 600), |phi| {
		Box::new(::views::menu_main::MainMenuView::new(phi))
	});
}
