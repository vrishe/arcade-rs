use phi::{Phi, View};
use phi::data::Rectangle;
use phi::gfx::{AlphaChannel, Renderable, Sprite};

use sdl2::pixels::Color;

use views::game::bullet::{Bullet, CannonType};


use super::CollisionBody;


const PLAYER_SPEED: f64 = 180.0;
const PLAYER_W: f64 = 64.0;
const PLAYER_H: f64 = 64.0;


/// The different states our Player might be in. In the image, they're ordered
/// from left to right, then from top to bottom.
#[derive(Clone, Copy)]
pub enum PlayerFrame {
	UpNorm   = 1,
	UpFast   = 2,
	UpSlow   = 0,
	MidNorm  = 4,
	MidFast  = 5,
	MidSlow  = 3,
	DownNorm = 7,
	DownFast = 8,
	DownSlow = 6
}

pub struct Player {
	rect: Rectangle,
	sprites: Vec<Sprite>,

	alpha: AlphaChannel,

	cannon: CannonType,
	current: PlayerFrame,
}

impl Player {
	pub fn new(phi: &mut Phi) -> Player {
		let (alpha, spritesheet) = super::load_spritesheet_with_alpha(phi, "assets/sprites/spaceship.png", 0.5).unwrap();
		//? When we know in advance how many elements the `Vec` we contain, we
		//? can allocate the good amount of data up-front.
		let mut sprites = Vec::with_capacity(9);

		let (w, h) = spritesheet.size();
		let w = w / 3.0;
		let h = h / 3.0;

		for y in 0..3 {
			for x in 0..3 {
				sprites.push(spritesheet.region(Rectangle {
					w: w,
					h: h,
					x: w * x as f64,
					y: h * y as f64,
				}).unwrap());
			}
		}
		Player {
			rect: Rectangle {
				x: 64.0,
				y: 64.0,
				w: PLAYER_W,
				h: PLAYER_H,
			},
			sprites: sprites,

			alpha: alpha,

			//? Let `RectBullet` be the default kind of bullet.
			cannon: CannonType::RectBullet,
			current: PlayerFrame::MidNorm,
		}
	}


	pub fn spawn_bullets(&self) -> Vec<Box<Bullet>> {
		let cannons_x = self.rect.x + 30.0;
		let cannon1_y = self.rect.y + 6.0;
		let cannon2_y = self.rect.y + PLAYER_H - 10.0;

		super::bullet::spawn_bullets(self.cannon, cannons_x, cannon1_y, cannon2_y)
	}


	pub fn update(&mut self, phi: &mut Phi, elapsed: f64) {
		// Change the player's cannons

		if phi.events.now.key_1 == Some(true) {
			self.cannon = CannonType::RectBullet;
		}

		if phi.events.now.key_2 == Some(true) {
			self.cannon = CannonType::SineBullet {
				amplitude: 10.0,
				angular_vel: 15.0,
			};
		}

		if phi.events.now.key_3 == Some(true) {
			self.cannon = CannonType::DivergentBullet {
				a: 100.0,
				b: 1.2,
			};
		}

		// Move the player's ship

		let diagonal =
		(phi.events.key_up ^ phi.events.key_down) &&
		(phi.events.key_left ^ phi.events.key_right);

		let moved =
		if diagonal { 1.0 / 2.0f64.sqrt() }
		else { 1.0 } * PLAYER_SPEED * elapsed;

		let dx = match (phi.events.key_left, phi.events.key_right) {
			(true, true) | (false, false) => 0.0,
			(true, false) => -moved,
			(false, true) => moved,
		};

		let dy = match (phi.events.key_up, phi.events.key_down) {
			(true, true) | (false, false) => 0.0,
			(true, false) => -moved,
			(false, true) => moved,
		};

		self.rect.x += dx;
		self.rect.y += dy;

		// The movable region spans the entire height of the window and 70% of its
		// width. This way, the player cannot get to the far right of the screen, where
		// we will spawn the asteroids, and get immediately eliminated.
		//
		// We restrain the width because most screens are wider than they are high.
		let movable_region = Rectangle {
			x: 0.0,
			y: 0.0,
			w: phi.output_size().0 as f64 * 0.70,
			h: phi.output_size().1 as f64,
		};

		// If the player cannot fit in the screen, then there is a problem and
		// the game should be promptly aborted.
		self.rect = self.rect.move_inside(movable_region).unwrap();

		// Select the appropriate sprite of the ship to show.
		self.current =
		if dx == 0.0 && dy < 0.0       { PlayerFrame::UpNorm }
		else if dx > 0.0 && dy < 0.0   { PlayerFrame::UpFast }
		else if dx < 0.0 && dy < 0.0   { PlayerFrame::UpSlow }
		else if dx == 0.0 && dy == 0.0 { PlayerFrame::MidNorm }
		else if dx > 0.0 && dy == 0.0  { PlayerFrame::MidFast }
		else if dx < 0.0 && dy == 0.0  { PlayerFrame::MidSlow }
		else if dx == 0.0 && dy > 0.0  { PlayerFrame::DownNorm }
		else if dx > 0.0 && dy > 0.0   { PlayerFrame::DownFast }
		else if dx < 0.0 && dy > 0.0   { PlayerFrame::DownSlow }
		else { unreachable!() };
	}

	pub fn render(&self, phi: &mut Phi) {
		// Render the bounding box (for debugging purposes)
		if ::DEBUG {
			phi.renderer.set_draw_color(Color::RGB(10, 200, 50));
			phi.renderer.fill_rect(self.rect.to_sdl().unwrap()).unwrap();
		}

		// Render the ship's current sprite.
		self.sprites[self.current as usize]
		.render(&mut phi.renderer, self.rect);
	}
}

impl CollisionBody for Player {
	fn rect(&self) -> &Rectangle {
		&self.rect
	}

	fn frame(&self) -> Rectangle {
		self.sprites[self.current as usize].frame()
	}

	fn alpha(&self) -> &AlphaChannel {
		&self.alpha
	}
}