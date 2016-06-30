extern crate sdl2_sys;

use phi::data::Rectangle;
use phi::Phi;

use sdl2::render::{Renderer, Texture};
use sdl2::surface::Surface;
use sdl2_image::LoadTexture;

use self::sdl2_sys::pixels as ll;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;


macro_rules! aligned (
	( $value_expr: expr ; $bound_expr: expr ) => { 
		{ let _bound = $bound_expr; (($value_expr) + _bound - 1) / _bound } 
	}
);

pub unsafe fn extract_alpha(surface: &Surface, threshold: Option<u8>) -> Vec<usize> {
	let pixel_format = surface.pixel_format().raw();

	match (*pixel_format).format {
		ll::SDL_PIXELFORMAT_ARGB4444 | ll::SDL_PIXELFORMAT_RGBA4444 | ll::SDL_PIXELFORMAT_ABGR4444 | ll::SDL_PIXELFORMAT_BGRA4444 | 
		ll::SDL_PIXELFORMAT_ARGB1555 | ll::SDL_PIXELFORMAT_RGBA5551 | ll::SDL_PIXELFORMAT_ABGR1555 | ll::SDL_PIXELFORMAT_BGRA5551 |
		ll::SDL_PIXELFORMAT_ARGB8888 | ll::SDL_PIXELFORMAT_RGBA8888 | ll::SDL_PIXELFORMAT_ABGR8888 | ll::SDL_PIXELFORMAT_BGRA8888 |
		ll::SDL_PIXELFORMAT_ARGB2101010 => {

			let threshold = threshold.unwrap_or(255u8);
			let read_alpha: Box<Fn(*const u8) -> u8> = match (*pixel_format).format {
				ll::SDL_PIXELFORMAT_ARGB4444 | ll::SDL_PIXELFORMAT_RGBA4444 | ll::SDL_PIXELFORMAT_ABGR4444 | ll::SDL_PIXELFORMAT_BGRA4444 | 
				ll::SDL_PIXELFORMAT_ARGB1555 | ll::SDL_PIXELFORMAT_RGBA5551 | ll::SDL_PIXELFORMAT_ABGR1555 | ll::SDL_PIXELFORMAT_BGRA5551 => {
					Box::new(|pixels: *const u8| {
						((*(pixels as *const u16) >> (*pixel_format).Ashift) << (*pixel_format).Aloss) as u8
					})
				},
				ll::SDL_PIXELFORMAT_ARGB8888 | ll::SDL_PIXELFORMAT_RGBA8888 | ll::SDL_PIXELFORMAT_ABGR8888 | ll::SDL_PIXELFORMAT_BGRA8888 |
				ll::SDL_PIXELFORMAT_ARGB2101010 => {
					Box::new(|pixels: *const u8| { 
						((*(pixels as *const u32) >> (*pixel_format).Ashift) << (*pixel_format).Aloss) as u8
					})
				},
				_ => unreachable!()
			};
			let read_pixels = |pixels: &[u8]| {
				let size_usize = ::std::mem::size_of::<usize>(); // bytes
				let size_packed = aligned!(aligned!((surface.width() * surface.height()) as usize; 8); size_usize);

				let mut result: Vec<usize> = vec![0; size_packed];
				{
					let result_mutable = &mut result;
					let pixels_ptr = &pixels[0] as *const u8;
					let size_usize = size_usize as usize * 8; // bits
					let color_depth = (*pixel_format).BytesPerPixel as u32;
					
					for y in 0..surface.height() {
						let stride = y * surface.width();
						let row = pixels_ptr.offset((y * surface.pitch()) as isize);
						
						for x in 0..surface.width() {
							if read_alpha(row.offset((x * color_depth) as isize)) >= threshold {
								let i = (stride + x) as usize;

								result_mutable[i / size_usize] |= 1usize << (i % size_usize);
							}
						}
					}					
				}
				result	
			};
			match surface.without_lock() {
				Some(pixels) => read_pixels(pixels),
				None => surface.with_lock(read_pixels)
			}	
		},
		_ => panic!("Surface does not contain alpha channel!")
	}
}

/// Common interface for rendering a graphical component to some given region
/// of the window.
pub trait Renderable {
	fn render(&self, renderer: &mut Renderer, dest: Rectangle);
}


#[derive(Clone)]
pub struct Sprite {
	alpha: Option<Rc<RefCell<Vec<usize>>>>,

	tex: Rc<RefCell<Texture>>,
	src: Rectangle,
}


impl Sprite {
	/// Creates a new sprite by wrapping a `Texture`.
	fn new(texture: Texture, alpha: Option<Vec<usize>>) -> Sprite {
		let tex_query = texture.query();

		Sprite {
			alpha: alpha.map_or(None, |v| Some(Rc::new(RefCell::new(v)))),

			tex: Rc::new(RefCell::new(texture)),
			src: Rectangle {
				w: tex_query.width as f64,
				h: tex_query.height as f64,
				x: 0.0,
				y: 0.0,
			}
		}
	}


	pub fn from_texture(texture: Texture) -> Sprite {
		Sprite::new(texture, None)
	}


	/// Creates a new sprite from an image file located at the given path.
	/// Returns `Some` if the file could be read, and `None` otherwise.
	pub fn load(renderer: &Renderer, path: &str) -> Option<Sprite> {
		renderer.load_texture(Path::new(path)).ok().map(|v| Sprite::new(v, None))
	}

	pub fn load_with_alpha(renderer: &Renderer, path: &str, alpha_threshold: u8) -> Option<Sprite> {
		use sdl2::rwops::RWops;
		use sdl2_image::ImageRWops;

		let surface_reader = RWops::from_file(path, "rb").unwrap();

		unsafe {
			let surface =surface_reader.load().unwrap();
			let alpha = Some(extract_alpha(&surface, Some(alpha_threshold)));

			renderer.create_texture_from_surface(surface).ok().map(|v| Sprite::new(v, alpha))	
		}
	}

	/// Returns a new `Sprite` representing a sub-region of the current one.
	/// The provided `rect` is relative to the currently held region.
	/// Returns `Some` if the `rect` is valid, i.e. included in the current
	/// region, and `None` otherwise.
	pub fn region(&self, rect: Rectangle) -> Option<Sprite> {
		let new_src = Rectangle {
			x: rect.x + self.src.x,
			y: rect.y + self.src.y,
			..rect
		};

		// Verify that the requested region is inside of the current one
		if self.src.contains(new_src) {
			return Some(Sprite {
				alpha: self.alpha.clone(),
				tex: self.tex.clone(),
				src: new_src,
			})
		}
		None
	}

	// Returns the dimensions of the region.
	pub fn size(&self) -> (f64, f64) {
		(self.src.w, self.src.h)
	}
}

impl Renderable for Sprite {
	fn render(&self, renderer: &mut Renderer, dest: Rectangle) {
		renderer.copy(&mut self.tex.borrow_mut(), self.src.to_sdl(), dest.to_sdl())
	}
}


#[derive(Clone)]
pub struct AnimatedSprite {
	/// The frames that will be rendered, in order.
	sprites: Rc<Vec<Sprite>>,

	/// The time it takes to get from one frame to the next, in seconds.
	frame_delay: f64,

	/// The total time that the sprite has been alive, from which the current
	/// frame is derived.
	current_time: f64,
	current_frame: usize,
}

pub struct AnimatedSpriteDescr<'a> {
	pub image_path: &'a str,
	pub total_frames: usize,
	pub frames_high: usize,
	pub frames_wide: usize,
	pub frame_w: f64,
	pub frame_h: f64,
}


impl AnimatedSprite {	
	/// Creates a new animated sprite initialized at time 0.
	pub fn new(sprites: Vec<Sprite>, frame_delay: f64) -> AnimatedSprite {
		AnimatedSprite {
			sprites: Rc::new(sprites),
			frame_delay: frame_delay,
			current_time: 0.0,
			current_frame: 0,
		}
	}


	pub fn load_frames(phi: &mut Phi, descr: AnimatedSpriteDescr) -> Vec<Sprite> {
		// Read the spritesheet image from the filesystem and construct an
		// animated sprite out of it.

		let spritesheet = Sprite::load(&mut phi.renderer, descr.image_path).unwrap();
		
		Self::load_frames_impl(&spritesheet, &descr)
	}

	pub fn load_frames_with_alpha(phi: &mut Phi, descr: AnimatedSpriteDescr, alpha_threshold: u8) -> Vec<Sprite> {
		// Read the spritesheet image from the filesystem and construct an
		// animated sprite out of it.

		let spritesheet = Sprite::load_with_alpha(&mut phi.renderer, descr.image_path, alpha_threshold).unwrap();
		
		Self::load_frames_impl(&spritesheet, &descr)
	}

	fn load_frames_impl(spritesheet: &Sprite, descr: &AnimatedSpriteDescr) -> Vec<Sprite> {
		let mut frames = Vec::with_capacity(descr.total_frames);

		for yth in 0..descr.frames_high {
			for xth in 0..descr.frames_wide {
				if descr.frames_wide * yth + xth < descr.total_frames {
					frames.push(
						spritesheet.region(Rectangle {
							w: descr.frame_w,
							h: descr.frame_h,
							x: descr.frame_w * xth as f64,
							y: descr.frame_h * yth as f64,
						}).unwrap());
				}
			}
		}
		frames		
	}


	/// Creates a new animated sprite which goes to the next frame `fps` times
	/// every second.
	pub fn with_fps(sprites: Vec<Sprite>, fps: f64) -> AnimatedSprite {
		//? Logically, a value of 0FPS might mean "stop changing frames".
		//? However, there's not really a need for this functionality in this
		//? game.
		//?
		//? If you would like to implement this functionality yourself, I would
		//? suggest that you add a `current_frame` attribute to `AnimatedSprite`
		//? which is used whenever `frame_delay`, in this scenario of type
		//? `Option<f64>`, is `None` (where `None` == infinity).
		//?
		//? Then, when `with_fps` or `set_fps` gets a value of 0, you compute
		//? the current frame and assign it to `current_frame`, then set
		//? `frame_delay` to `None`. The rest is yours to solve.
		if fps == 0.0 {
			panic!("Passed 0 to AnimatedSprite::with_fps");
		}
		AnimatedSprite::new(sprites, 1.0 / fps)
	}


	// The number of frames composing the animation.
	pub fn frames_count(&self) -> usize {
		self.sprites.len()
	}

	pub fn current_frame_index(&self) -> usize {
		self.current_frame
	}

	pub fn get_frame_at(&self, index: usize) -> &Sprite {
		&self.sprites[index]
	}


	/// Set the time it takes to get from one frame to the next, in seconds.
	/// If the value is negative, then we "rewind" the animation.
	pub fn set_frame_delay(&mut self, frame_delay: f64) {
		self.frame_delay = frame_delay;
	}

	/// Set the number of frames the animation goes through every second.
	/// If the value is negative, then we "rewind" the animation.
	pub fn set_fps(&mut self, fps: f64) {
		if fps == 0.0 {
			panic!("Passed 0 to AnimatedSprite::set_fps");
		}
		self.set_frame_delay(1.0 / fps);
	}

	/// Adds a certain amount of time, in seconds, to the `current_time` of the
	/// animated sprite, so that it knows when it must go to the next frame.
	pub fn add_time(&mut self, dt: f64) {
		self.current_time += dt;

		// If we decide to go "back in time", this allows us to select the
		// last frame whenever we reach a negative one.
		if self.current_time < 0.0 {
			self.current_time = (self.frames_count() - 1) as f64 * self.frame_delay;
		}
		self.current_frame = (self.current_time / self.frame_delay) as usize % self.frames_count();
	}
}

impl Renderable for AnimatedSprite {
	/// Renders the current frame of the sprite.
	fn render(&self, renderer: &mut Renderer, dest: Rectangle) {
		self.sprites[self.current_frame_index()].render(renderer, dest);
	}
}


pub trait CopySprite<T> {
	fn copy_sprite(&mut self, sprite: &T, dest: Rectangle);
}

impl<'window, T: Renderable> CopySprite<T> for Renderer<'window> {
	fn copy_sprite(&mut self, renderable: &T, dest: Rectangle) {
		renderable.render(self, dest);
	}
}