extern crate sdl2;

#[macro_use]
mod events;


use sdl2::pixels::Color;


struct_events!{
	keyboard: {
		key_escape: Escape,
		key_up: Up,
		key_down: Down
	},
	other: {
		quit: Quit { .. }
	}
}

fn main() {
	// SDL2 init
	let context = sdl2::init().unwrap();
	let video = context.video().unwrap();

	let window = video.window("ArcadeRS Shooter", 800, 600)
		.position_centered()
		.opengl()
		.build()
		.unwrap();

	let mut renderer = window.renderer()
		.accelerated()
		.build()
		.unwrap();


	let mut events = Events::new(context.event_pump().unwrap());

	loop {
		events.pump();

		if events.now.quit || events.now.key_escape == Some(true) {
			break;
		}
		renderer.set_draw_color(Color::RGB(0, 0, 0));
		renderer.clear();
		renderer.present();
	}
}
