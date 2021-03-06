
#[macro_use]
mod events;

pub mod data;
pub mod gfx;


use sdl2::pixels::Color;
use sdl2::render::Renderer;
use sdl2::ttf::{Sdl2TtfContext/*, Font*/};

// use std::collections::HashMap;
use std::path::Path;


use self::gfx::Sprite;


struct_events!{
	keyboard: {
		key_escape: Escape,
		key_return: Return,
		key_space: Space,

		key_down: Down,
		key_left: Left,
		key_right: Right,
		key_up: Up,

		key_1: Num1,
		key_2: Num2,
		key_3: Num3
	},
	other: {
		quit: Quit { .. }
	}
}


/// A `ViewAction` is a way for the currently executed view to
/// communicate with the game loop. It specifies which action
/// should be executed before the next rendering.
pub enum ViewAction {
	Render(Box<View>),
	Quit,
}


/// Bundles the Phi abstractions in a single structure which
/// can be passed easily between functions.
pub struct Phi<'window> {
	pub events: Events,
	pub renderer: Renderer<'window>,

	ttf_context: Sdl2TtfContext,
	// cached_fonts: HashMap<(&'static str, u16), Font<'window, 'static>>,

	allocated_channels: i32,
}

impl <'window> Phi<'window> {
	fn new(events: Events, ttf_context: Sdl2TtfContext, renderer: Renderer<'window>) -> Phi<'window> {
		let result = Phi {
			events: events,
			renderer: renderer,

			ttf_context: ttf_context,
			// cached_fonts: HashMap::new(),

			allocated_channels: 32
		};
		//? This function asks us how many channels we wish to allocate for our game.
		//? That is, how many sounds do we wish to be able to play at the same time?
		//? While testing, 16 channels seemed to be sufficient. Which means that we
		//? should probably request 32 of 'em just in case. :-°
		::sdl2::mixer::allocate_channels(result.allocated_channels);

		result
	}

	pub fn output_size(&self) -> (f64, f64) {
		let (w, h) = self.renderer.output_size().unwrap();
		(w as f64, h as f64)
	}

	pub fn ttf_str_sprite(&mut self, text: &str, font_path: &'static str, size: u16, color: Color) -> Option<Sprite> {
		// //? First, we verify whether the font is already cached. If this is the
		// //? case, we use it to render the text.
		// if let Some(font) = self.cached_fonts.get(&(font_path, size)) {
		// 	return font.render(text).blended(color).ok()
		// 		.and_then(|surface| self.renderer.create_texture_from_surface(&surface).ok())
		// 		.map(Sprite::from_texture)
		// }
		// //? Otherwise, we start by trying to load the requested font.
		// self.ttf_context.load_font(Path::new(font_path), size).ok()
		// 	.and_then(|font| {
		// 		//? If this works, we cache the font we acquired.
		// 		self.cached_fonts.insert((font_path, size), font);
		// 		//? Then, we call the method recursively. Because we know that
		// 		//? the font has been cached, the `if` block will be executed
		// 		//? and the sprite will be appropriately rendered.
		// 		self.ttf_str_sprite(text, font_path, size, color)
		// 	})

		self.ttf_context.load_font(Path::new(font_path), size).ok()
			.and_then(|font| font.render(text).blended(color).ok())
			.and_then(|surface| self.renderer.create_texture_from_surface(&surface).ok())
			.map(Sprite::from_texture)
	}

	/// Play a sound once, and allocate new channels if this is necessary.
	pub fn play_sound(&mut self, sound: &::sdl2::mixer::Chunk) {
		// Attempt to play the sound once.
		match ::sdl2::mixer::Channel::all().play(sound, 0) {
			Err(_) => {
				// If there weren't enough channels allocated, then we double
				// that number and try again.
				self.allocated_channels *= 2;
				::sdl2::mixer::allocate_channels(self.allocated_channels);

				self.play_sound(sound);
			},
			_ => { /* Everything's Alright! */ }
		}
	}
}

pub trait RendererExtensions {
	fn fill_circle(self: &mut Self, x: f64, y:f64, radius: f64) -> Result<(), String>;
}

impl <'window> RendererExtensions for Renderer<'window> {
	fn fill_circle(self: &mut Self, x: f64, y:f64, radius: f64) -> Result<(), String> {
		use sdl2::rect::Point as SdlPoint;

		let (mut e, mut a, mut b) = (0.0, radius, 0.0);

		while a >= b {
			let points = [
				SdlPoint::new((x + a) as i32, (y + b) as i32),
				SdlPoint::new((x + b) as i32, (y + a) as i32),
				SdlPoint::new((x - b) as i32, (y + a) as i32),
				SdlPoint::new((x - a) as i32, (y + b) as i32),
				SdlPoint::new((x - a) as i32, (y - b) as i32),
				SdlPoint::new((x - b) as i32, (y - a) as i32),
				SdlPoint::new((x + b) as i32, (y - a) as i32),
				SdlPoint::new((x + a) as i32, (y - b) as i32),
			];
			try!(self.draw_points(&points));

			b += 1.0;
			e += 1.0 + 2.0 * b;
			if 2.0 * (e - a) + 1.0 > 0.0 {
				a -= 1.0;
				e += 1.0 - 2.0 * a;
			}
		}
		Ok(())
	}
}


pub trait View {
	/// Called on every frame to take care of the logic of the program. From
	/// user inputs and the instance's internal state, determine whether to
	/// render itself or another view, close the window, etc.
    ///
    /// `elapsed` is expressed in seconds.
    fn update(self: Box<Self>, context: &mut Phi, elapsed: f64) -> ViewAction;

    /// Called on every frame to take care rendering the current view. It
    /// disallows mutating the object by default, although you may still do it
    /// through a `RefCell` if you need to.
    fn render(&self, context: &mut Phi);
}


/// Create a window with name `title`, initialize the underlying libraries and
/// start the game with the `View` returned by `init()`.
pub fn spawn<F>(title: &str, size: (u32, u32), init: F) where F: Fn(&mut Phi) -> Box<View> {
	// Initialize SDL2
	let sdl_context = ::sdl2::init().unwrap();
	let video = sdl_context.video().unwrap();
	let mut timer = sdl_context.timer().unwrap();
	let _image_context = ::sdl2::image::init(::sdl2::image::INIT_PNG).unwrap();
	let _ttf_context = ::sdl2::ttf::init().unwrap();

	// Initialize audio plugin
	//? We will stick to the Ogg format throughout this article. However, you
	//? can easily require other ones.
	let _mixer_context = ::sdl2::mixer::init(::sdl2::mixer::INIT_OGG).unwrap();
	//? We configure our audio context so that:
	//?   * The frequency is 44100;
	//?   * Use signed 16 bits samples, in little-endian byte order;
	//?   * It's also stereo (2 "channels");
	//?   * Samples are 1024 bytes in size.
	//? You don't really need to understand what all of this means. I myself just
	//? copy-pasted this from andelf's demo. ;-)
	::sdl2::mixer::open_audio(44100, ::sdl2::mixer::AUDIO_S16LSB, 2, 1024).unwrap();

	// Create the window
	let window = video.window(title, size.0, size.1)
	.position_centered()
	.opengl()
	// .resizable()
	.build().unwrap();

	// Create the context
	let mut context = Phi::new(
		Events::new(sdl_context.event_pump().unwrap()), 
		_ttf_context,
		window.renderer()
		.accelerated()
		.build().unwrap());

	// Create the default view
	let mut current_view = init(&mut context);


	// Frame timing

	let interval = 1_000 / 60;
	let mut before = timer.ticks();
	let mut last_second = timer.ticks();
	let mut fps = 0u16;

	loop {
		// Frame timing (bis)

		let now = timer.ticks();
		let dt = now - before;
		let elapsed = dt as f64 / 1_000.0;

		// If the time elapsed since the last frame is too small, wait out the
		// difference and try again.
		if dt < interval {
			timer.delay(interval - dt);
			continue;
		}

		before = now;
		fps += 1;

		if now - last_second > 1_000 {
			println!("FPS: {}", fps);
			last_second = now;
			fps = 0;
		}
		// Logic & rendering
		context.events.pump(&mut context.renderer);

		match current_view.update(&mut context, elapsed) {
			ViewAction::Render(view) => {
				current_view = view;
				current_view.render(&mut context);
				context.renderer.present();
			},
			ViewAction::Quit => break,
		}
	}
}