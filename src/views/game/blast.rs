use phi::{Phi, RendererExtensions};

use sdl2::pixels::Color;


use super::GameObject;


const BLAST_RADIUS_MAX: f64 = 53.568;
const BLAST_RADIUS_MIN: f64 = 32.448;
const BLAST_DURATION: f64 = 121.3;


pub struct Blast {
	center: (f64, f64),

	blast_radius: f64,
	lifetime: f64,
}


impl Blast {
	pub fn new(center: (f64, f64)) -> Blast {
		Blast {
			center: center,

			blast_radius: BLAST_RADIUS_MAX,
			lifetime: 0.0,
		}
	}

	pub fn hits_at(&self, location: (f64, f64)) -> bool {
		let location = (location.0 - self.center.0, location.1 - self.center.1);

		return (location.0 * location.0 + location.1 * location.1) < self.blast_radius * self.blast_radius;			
	}
}


impl GameObject<Blast> for Blast {

	fn is_alive(&self) -> bool {
		self.lifetime < BLAST_DURATION
	}

	fn location(&self) -> (f64, f64) {
		self.center
	}

	fn update(mut self: Box<Blast>, _context: &mut Phi, dt: f64) -> Option<Box<Blast>> {
		self.lifetime += dt;

		if self.is_alive() {
			let t = (1000.0 * self.lifetime) / BLAST_DURATION;
			let value = BLAST_RADIUS_MAX - (BLAST_RADIUS_MAX - BLAST_RADIUS_MIN) * (t * t * t * t * t);

			self.blast_radius = value * value;

			return Some(self);
		}
		None		
	}

	fn render(&self, context: &mut Phi) {
		assert!(self.is_alive());
		
		context.renderer.set_draw_color(Color::RGB(200, 50, 10));
		context.renderer.fill_circle(self.center.0, self.center.1, self.blast_radius.sqrt()).unwrap();	
	}
}