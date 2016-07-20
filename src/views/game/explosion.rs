use phi::{Phi, View};
use phi::data::Rectangle;
use phi::gfx::{AnimatedSprite, AnimatedSpriteDescr, Renderable};


use super::GameObject;


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
	lifetime: f64,
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
}

impl GameObject<Explosion> for Explosion {

	fn is_alive(&self) -> bool {
		self.lifetime <= EXPLOSION_DURATION
	}

	fn location(&self) -> (f64, f64) {
		self.rect.location()
	}

	fn update(mut self: Box<Explosion>, _: &mut Phi, dt: f64) -> Option<Box<Explosion>> {
		self.lifetime += dt;
		self.sprite.add_time(dt);

		if self.is_alive() {
			return Some(self);
		}
		None
	}

	fn render(&self, context: &mut Phi) {
		self.sprite.render(&mut context.renderer, self.rect);
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

			lifetime: 0.0,
		}
	}
}