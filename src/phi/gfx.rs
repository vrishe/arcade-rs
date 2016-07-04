extern crate sdl2_sys;

use phi::data::Rectangle;
use phi::Phi;

use sdl2::render::{Renderer, Texture};
use sdl2::surface::Surface;
use sdl2_image::LoadTexture;

use self::sdl2_sys::pixels as ll;

use std::cell::RefCell;
use std::ops::{ Index, Range };
use std::path::Path;
use std::rc::Rc;


macro_rules! aligned (
	( $value_expr: expr ; $bound_expr: expr ) => { 
		{ let _bound = $bound_expr; (($value_expr) + _bound - 1) / _bound } 
	}
);


#[derive(Clone, Debug)]
pub struct AlphaChannel {
	data: Vec<usize>,

	stride: usize,
	
	height: u32,
	width: u32,	
}

pub trait Collide {
	fn collide(channel_a: &AlphaChannel, x_a: f64, y_a: f64, channel_b: &AlphaChannel, x_b: f64, y_b: f64, roi: Rectangle) -> bool;
}

pub trait BoxCollide {
	fn collide(channel: &AlphaChannel, x: f64, y: f64, roi: Rectangle) -> bool;	
}

impl AlphaChannel {

	pub unsafe fn from_surface(surface: &Surface, threshold: Option<u8>) -> Option<AlphaChannel> {
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
					let size_usize = ::std::mem::size_of::<usize>() * 8; // bytes
					let size_packed = aligned!(surface.width() as usize; size_usize);

					let mut result: Vec<usize> = vec![0; size_packed * surface.height() as usize];
					{
						let result_mutable = &mut result;
						let pixels_ptr = &pixels[0] as *const u8;
						let color_depth = (*pixel_format).BytesPerPixel as usize;

						for y in 0usize..surface.height() as usize {
							let stride = y * size_packed;
							let row = pixels_ptr.offset((y * surface.pitch() as usize) as isize);

							for x in 0usize..surface.width() as usize {
								if read_alpha(row.offset((x * color_depth) as isize)) >= threshold {
									result_mutable[stride + x / size_usize] |= 1usize << (x % size_usize);
								}
							}
						}					
					}
					AlphaChannel {
						data: result,

						stride: size_packed,

						height: surface.height(),
						width: surface.width(),				
					}
				};
				Some(match surface.without_lock() {
					Some(pixels) => read_pixels(pixels),
					None => surface.with_lock(read_pixels)
				})
			},
			_ => None
		}
	}


	pub fn stride(&self) -> usize {
		self.stride
	}

	pub fn width(&self) -> u32 {
		self.width
	}

	pub fn height(&self) -> u32 {
		self.height
	}

	pub fn size(&self) -> (u32, u32) {
		(self.width, self.height)
	}
}

impl Index<usize> for AlphaChannel {
	type Output = usize;

	fn index<'a>(&'a self, _index: usize) -> &'a usize {
		&self.data[_index]
	}
}

impl Index<Range<usize>> for AlphaChannel {
	type Output = [usize];

	fn index<'a>(&'a self, _index: Range<usize>) -> &'a [usize] {
		&self.data[_index]
	}
}

impl Collide for AlphaChannel {

	fn collide(channel_a: &AlphaChannel, x_a: f64, y_a: f64, channel_b: &AlphaChannel, x_b: f64, y_b: f64, roi: Rectangle) -> bool {
		let (w, h) = (roi.w.round() as usize, roi.h.round() as usize);

		if w > 0 {
			let (x_a, x_b) = ((roi.x - x_a).round() as usize, (roi.x - x_b).round() as usize);
			let (y_a, y_b) = ((roi.y - y_a).round() as usize, (roi.y - y_b).round() as usize);
			let (xlast_a, xlast_b) = (x_a + w - 1, x_b + w - 1);

			let size_usize = ::std::mem::size_of::<usize>() * 8;

			let (maskl_a, maskl_b) = (
				::std::usize::MAX.wrapping_shl((x_a % size_usize) as u32),
				::std::usize::MAX.wrapping_shl((x_b % size_usize) as u32));			
			let (maskr_a, maskr_b) = (
				1usize.wrapping_shl((xlast_a % size_usize) as u32).wrapping_sub(1),
				1usize.wrapping_shl((xlast_b % size_usize) as u32).wrapping_sub(1));

			let w = aligned!(w; size_usize);
			let (x_a, x_b) = (x_a / size_usize, x_b / size_usize);
			let (w_a, w_b) = (
				xlast_a / size_usize - x_a, 
				xlast_b / size_usize - x_b);

			let get_block = |r: usize, channel: &AlphaChannel, offset: usize, last: usize, maskl: usize, maskr: usize| {
				let i = offset + r;
				let mut result = channel[i];

				if r <= 0 {
					result &= maskl;
				}
				if r >= last {
					result &= maskr;
				}
				return result;
			};
			for r in 0..h {
				let (row_a, row_b) = ((y_a + r) * channel_a.stride() + x_a, (y_b + r) * channel_b.stride() + x_b);

				for c in 0..w {
					let (block_a, block_b) = (
						get_block(c, channel_a, row_a, w_a, maskl_a, maskr_a),
						get_block(c, channel_b, row_b, w_b, maskl_b, maskr_b));

					if (block_a & block_b) != 0 {
						return true;
					}
				}
			}
		}
		false
	}
}

impl BoxCollide for AlphaChannel {

	fn collide(channel: &AlphaChannel, x: f64, y: f64, roi: Rectangle) -> bool {
		let (w, h) = (roi.w.round() as usize, roi.h.round() as usize);

		if w > 0 {
			let (x, y) = ((roi.x - x).round() as usize, (roi.y - y).round() as usize);
			
			let xlast = x + w;

			let size_usize = ::std::mem::size_of::<usize>() * 8;
			let maskl = ::std::usize::MAX << (x % size_usize);
			let maskr = !::std::usize::MAX.wrapping_shl((xlast % size_usize + 1) as u32);

			let x = x / size_usize;
			let w = xlast / size_usize - x + 1;

			if w > 0 {
				let rlast = w - 1;

				for c in 0..h {
					let row = (c + y) * channel.stride() + x;

					for r in 0..w {
						let mut block = channel[row + r];

						if r == 0 { block &= maskl; }
						if r == rlast { block &= maskr; }

						if block != 0 {
							return true;
						}
					}			
				}			
			}			
		}
		false
	}
}


/// Common interface for rendering a graphical component to some given region
/// of the window.
pub trait Renderable {
	fn render(&self, renderer: &mut Renderer, dest: Rectangle);
}


#[derive(Clone)]
pub struct Sprite {
	tex: Rc<RefCell<Texture>>,
	src: Rectangle,
}


impl Sprite {
	/// Creates a new sprite by wrapping a `Texture`.
	fn new(texture: Texture) -> Sprite {
		let tex_query = texture.query();

		Sprite {
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
		Sprite::new(texture)
	}

	pub fn from_surface(renderer: &Renderer, surface: &Surface) -> Option<Sprite> {
		renderer.create_texture_from_surface(surface).ok().map(Sprite::new)
	}


	/// Creates a new sprite from an image file located at the given path.
	/// Returns `Some` if the file could be read, and `None` otherwise.
	pub fn load(renderer: &Renderer, path: &str) -> Option<Sprite> {
		renderer.load_texture(Path::new(path)).ok().map(Sprite::new)
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

	pub fn frame(&self) -> Rectangle {
		self.src
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

pub struct AnimatedSpriteDescr {
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

	pub fn load(path: &str, phi: &mut Phi, descr: AnimatedSpriteDescr) -> AnimatedSprite {
		let spritesheet = Sprite::load(&mut phi.renderer, path).unwrap();

		AnimatedSprite::new(Self::load_frames(&spritesheet, descr), 0.0)
	}

	pub fn load_with_fps(path: &str, phi: &mut Phi, descr: AnimatedSpriteDescr, fps: f64) -> AnimatedSprite {
		if fps == 0.0 {
			panic!("Passed 0 to AnimatedSprite::with_fps");
		}
		let spritesheet = Sprite::load(&mut phi.renderer, path).unwrap();

		AnimatedSprite::new(Self::load_frames(&spritesheet, descr), 1f64 / fps)
	}


	pub fn load_frames(spritesheet: &Sprite, descr: AnimatedSpriteDescr) -> Vec<Sprite> {
		// Read the spritesheet image from the filesystem and construct an
		// animated sprite out of it.
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