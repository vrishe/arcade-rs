
#[macro_use]
mod events;

pub mod data;
pub mod gfx;


use self::gfx::Sprite;

use sdl2::pixels::Color;
use sdl2::render::Renderer;
use sdl2_ttf::{Sdl2TtfContext, Font};

use std::collections::HashMap;
use std::path::Path;


struct_events!{
	keyboard: {
		key_escape: Escape,
		key_return: Return,
		key_space: Space,

		key_down: Down,
		key_left: Left,
		key_right: Right,
		key_up: Up
	},
	other: {
		quit: Quit { .. }
	}
}


/// A `ViewAction` is a way for the currently executed view to
/// communicate with the game loop. It specifies which action
/// should be executed before the next rendering.
pub enum ViewAction {
	ChangeView(Box<View>),
	None,
	Quit,
}


/// Bundles the Phi abstractions in a single structure which
/// can be passed easily between functions.
pub struct Phi<'window> {
	pub events: Events,
	pub renderer: Renderer<'window>,

	ttf_context: Sdl2TtfContext,
	cached_fonts: HashMap<(&'static str, u16), Font>,
}

impl <'window> Phi<'window> {
	fn new(events: Events, ttf_context: Sdl2TtfContext, renderer: Renderer<'window>) -> Phi<'window> {
		Phi {
			events: events,
			renderer: renderer,

			ttf_context: ttf_context,
			cached_fonts: HashMap::new(),
		}
	}

	pub fn output_size(&self) -> (f64, f64) {
		let (w, h) = self.renderer.output_size().unwrap();
		(w as f64, h as f64)
	}

	pub fn ttf_str_sprite(&mut self, text: &str, font_path: &'static str, size: u16, color: Color) -> Option<Sprite> {
		//? First, we verify whether the font is already cached. If this is the
		//? case, we use it to render the text.
		if let Some(font) = self.cached_fonts.get(&(font_path, size)) {
			return font.render(text).blended(color).ok()
			.and_then(|surface| self.renderer.create_texture_from_surface(&surface).ok())
			.map(Sprite::new)
		}
		//? Otherwise, we start by trying to load the requested font.
		self.ttf_context.load_font(Path::new(font_path), size).ok()
		.and_then(|font| {
			//? If this works, we cache the font we acquired.
			self.cached_fonts.insert((font_path, size), font);
			//? Then, we call the method recursively. Because we know that
			//? the font has been cached, the `if` block will be executed
			//? and the sprite will be appropriately rendered.
			self.ttf_str_sprite(text, font_path, size, color)
		})
	}
}


pub trait View {
	/// Called on every frame to take care of both the logic and
	/// the rendering of the current view.
	///
	/// `elapsed` is expressed in seconds.
	fn render(&mut self, context: &mut Phi, elapsed: f64) -> ViewAction;
}


/// Create a window with name `title`, initialize the underlying libraries and
/// start the game with the `View` returned by `init()`.
///
/// # Examples
///
/// Here, we simply show a window with color #ffff00 and exit when escape is
/// pressed or when the window is closed.
///
/// ```
/// struct MyView;
///
/// impl View for MyView {
///     fn render(&mut self, context: &mut Phi, _: f64) -> ViewAction {
///         if context.events.now.quit {
///             return ViewAction::Quit;
///         }
///
///         context.renderer.set_draw_color(Color::RGB(255, 255, 0));
///         context.renderer.clear();
///         ViewAction::None
///     }
/// }
///
/// spawn("Example", |_| {
///     Box::new(MyView)
/// });
/// ```
pub fn spawn<F>(title: &str, init: F)
where F: Fn(&mut Phi) -> Box<View> {
	// Initialize SDL2
	let sdl_context = ::sdl2::init().unwrap();
	let video = sdl_context.video().unwrap();
	let mut timer = sdl_context.timer().unwrap();
	let _image_context = ::sdl2_image::init(::sdl2_image::INIT_PNG).unwrap();
	let _ttf_context = ::sdl2_ttf::init().unwrap();

	// Create the window
	let window = video.window(title, 800, 600)
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

		match current_view.render(&mut context, elapsed) {
			ViewAction::ChangeView(new_view) => current_view = new_view,
			ViewAction::None => context.renderer.present(),
			ViewAction::Quit => break,
		}
	}
}