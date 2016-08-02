use phi::data::Rectangle;
use phi::gfx::{Renderable, Sprite};

use sdl2::render::Renderer;


#[derive(Clone)]
pub struct Button {
	frame: Rectangle,

	state: usize,
	sprites: Vec<Sprite>,
}

impl Button {

	pub fn load(renderer: &Renderer, path: &str, size: (f64, f64)) -> Option<Button> {
		let spritesheet = tryo!(Sprite::load(renderer, path));
		let spritesheet_size = spritesheet.size();
		let cols = spritesheet_size.0 / size.0;
		let rows = spritesheet_size.1 / size.1;

		if cols == ((cols as i64) as f64) 
		&& rows == ((rows as i64) as f64) {
			let mut sprites = Vec::with_capacity((rows * cols) as usize);

			for r in 0..rows as usize {
				for c in 0..cols as usize {
					sprites.push(tryo!(spritesheet.region(Rectangle {
						x: c as f64 * size.0,
						y: r as f64 * size.1,
						w: size.0,
						h: size.1
					})));
				}
			}
			return Some(Button {
				frame: Rectangle {
					x: 0.0,
					y: 0.0,
					w: size.0,
					h: size.1,					
				},
				state: 0,
				sprites: sprites,
			})
		}
		None
	}


	pub fn get_alpha(&self) -> f64 {
		self.sprites[self.state].get_alpha()
	}

	pub fn set_alpha(&mut self, alpha: f64) {
		for sprite in &mut self.sprites {
			sprite.set_alpha(alpha);
		}
	}

	pub fn get_state(&self) -> usize {
		self.state
	}

	pub fn set_state(&mut self, state: usize) {
		if state >= self.sprites.len() {
			panic!("state insdex is out of range!");
		}
		self.state = state;
	}

	pub fn set_location(&mut self, x: f64, y: f64) {
		self.frame.x = x;
		self.frame.y = y;
	}


	// Returns the dimensions of the region.
	pub fn size(&self) -> (f64, f64) {
		(self.frame.w, self.frame.h)
	}

	pub fn frame(&self) -> &Rectangle {
		&self.frame
	}


	pub fn render(&self, renderer: &mut Renderer) {
		self.sprites[self.state].render(renderer, self.frame);
	}	
}

