use phi::{Phi, View};
use phi::data::Rectangle;
use phi::gfx::{AlphaChannel, AnimatedSprite, AnimatedSpriteDescr, Renderable};

use sdl2::pixels::Color;

use std::rc::Rc;


use super::CollisionBody;


const ASTEROIDS_WIDE: usize = 21;
const ASTEROIDS_HIGH: usize = 7;
const ASTEROIDS_TOTAL: usize = ASTEROIDS_WIDE * ASTEROIDS_HIGH - 4;
const ASTEROID_SIDE: f64 = 96.0;


pub struct Asteroid {
	rect: Rectangle,
	sprite: AnimatedSprite,

	alpha: AlphaChannel,

	vel: f64,
}

impl Asteroid {
	pub fn factory(phi: &mut Phi) -> AsteroidFactory {
		let (alpha, spritesheet) = super::load_spritesheet_with_alpha(phi, "assets/sprites/asteroid.png", 0.5).unwrap();

		AsteroidFactory {
			alpha: Rc::new(alpha),
			sprite: AnimatedSprite::new(
				AnimatedSprite::load_frames(
					&spritesheet, 
					AnimatedSpriteDescr {
						total_frames: ASTEROIDS_TOTAL,
						frames_high: ASTEROIDS_HIGH,
						frames_wide: ASTEROIDS_WIDE,
						frame_w: ASTEROID_SIDE,
						frame_h: ASTEROID_SIDE,
					}),
				0.0),
		}
	}

	pub fn update(mut self, dt: f64) -> Option<Asteroid> {
		self.rect.x -= dt * self.vel;
		self.sprite.add_time(dt);

		if self.rect.x > -ASTEROID_SIDE {
			return Some(self)
		}
		None
	}

	pub fn render(&self, phi: &mut Phi) {
		if ::DEBUG {
			// Render the bounding box
			phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
			phi.renderer.fill_rect(self.rect.to_sdl().unwrap()).unwrap();
		}
		self.sprite.render(&mut phi.renderer, self.rect);
	}
}

impl<'a> CollisionBody for Asteroid {
	fn rect(&self) -> &Rectangle {
		&self.rect
	}

	fn frame(&self) -> Rectangle {
		self.sprite.get_frame_at(self.sprite.current_frame_index()).frame()
	}

	fn alpha(&self) -> &AlphaChannel {
		&self.alpha
	}
}


pub struct AsteroidFactory {
	alpha: Rc<AlphaChannel>,
	sprite: AnimatedSprite,
}

impl AsteroidFactory {
	pub fn random(&self, phi: &mut Phi) -> Asteroid {
		let (w, h) = phi.output_size();

		// FPS in [10.0, 30.0)
		let mut sprite = self.sprite.clone();

		sprite.set_fps(super::rand::random::<f64>().abs() * 20.0 + 10.0);

		Asteroid {
			// In the screen vertically, and over the right of the screen
			// horizontally.
			rect: Rectangle {
				w: ASTEROID_SIDE,
				h: ASTEROID_SIDE,
				x: w,
				y: super::rand::random::<f64>().abs() * (h - ASTEROID_SIDE),
			},
			sprite: sprite,

			alpha: self.alpha.as_ref().clone(),

			// vel in [50.0, 150.0)
			vel: super::rand::random::<f64>().abs() * 100.0 + 50.0,
		}
	}
}