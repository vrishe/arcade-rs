use phi::{Phi, RendererExtensions, View};
use phi::data::Rectangle;
use phi::gfx::{AnimatedSprite, AnimatedSpriteDescr, Renderable};

use sdl2::pixels::Color;


const EXPLOSIONS_WIDE: usize = 5;
const EXPLOSIONS_HIGH: usize = 4;
const EXPLOSIONS_TOTAL: usize = 17;
const EXPLOSION_SIDE: f64 = 96.0;
const EXPLOSION_FPS: f64 = 16.0;
const EXPLOSION_DURATION: f64 = 1.0 / EXPLOSION_FPS * EXPLOSIONS_TOTAL as f64;

const BLAST_RADIUS_MAX: f64 = EXPLOSION_SIDE * 0.558;
const BLAST_RADIUS_MIN: f64 = EXPLOSION_SIDE * 0.338;
const BLAST_DURATION: f64 = EXPLOSION_DURATION / 8.76;


pub struct Explosion {
	rect: Rectangle,
	sprite: AnimatedSprite,

	//? Keep how long its been arrived, so that we destroy the explosion once
	//? its animation is finished.
	alive_since: f64,
	blast_radius: f64,
}

impl Explosion {
	pub fn factory(phi: &mut Phi) -> ExplosionFactory {
		ExplosionFactory {
			sprite: AnimatedSprite::load_with_fps(
				"assets/sprites/explosion.png", phi,
				AnimatedSpriteDescr {
					total_frames: EXPLOSIONS_TOTAL,
					frames_high: EXPLOSIONS_HIGH,
					frames_wide: EXPLOSIONS_WIDE,
					frame_w: EXPLOSION_SIDE,
					frame_h: EXPLOSION_SIDE,
				},
				EXPLOSION_FPS),
		}
	}

	pub fn update(mut self, dt: f64) -> Option<Explosion> {
		self.alive_since += dt;
		self.sprite.add_time(dt);

		if self.alive_since < EXPLOSION_DURATION {
			if self.alive_since < BLAST_DURATION {
				let t = self.alive_since / BLAST_DURATION;
				let value = BLAST_RADIUS_MAX - (BLAST_RADIUS_MAX - BLAST_RADIUS_MIN) * (t * t * t * t * t);

				self.blast_radius = value * value;
			} else {
				self.blast_radius = -1.0;
			}
			return Some(self);
		}
		None
	}

	pub fn render(&self, phi: &mut Phi) {
		// Render the bounding box (for debugging purposes)
		if ::DEBUG {
			if self.blast_radius > 0.0 {
				let center = self.rect.center();

				phi.renderer.set_draw_color(Color::RGB(200, 50, 10));
				phi.renderer.fill_circle(center.0, center.1, self.blast_radius.sqrt()).unwrap();				
			}
		}
		self.sprite.render(&mut phi.renderer, self.rect);
	}


	pub fn blast(&self, location: (f64, f64)) -> bool {
		if self.blast_radius >= 0.0 {
			let center = self.rect.center();
			let vector = (location.0 - center.0, location.1 - center.1);

			return vector.0 * vector.0 + vector.1 * vector.1 <= self.blast_radius;			
		}
		false
	}
}


pub struct ExplosionFactory {
	sprite: AnimatedSprite,
}

impl ExplosionFactory {
	pub fn at_center(&self, center: (f64, f64)) -> Explosion {
		// FPS in [10.0, 30.0)
		let sprite = self.sprite.clone();

		Explosion {
			sprite: sprite,

			// In the screen vertically, and over the right of the screen
			// horizontally.
			rect: Rectangle::with_size(EXPLOSION_SIDE, EXPLOSION_SIDE)
			.center_at(center),

			alive_since: 0.0,
			blast_radius: -1.0,
		}
	}
}