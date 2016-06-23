extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;


mod phi;
mod views;


fn main() {
	::phi::spawn("ArcadeRS Shooter", |phi| {
        Box::new(::views::menu_main::MainMenuView::new(phi))
    });
}
