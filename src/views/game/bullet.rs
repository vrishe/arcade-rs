use phi::Phi;
use phi::data::Rectangle;
use phi::gfx::AlphaChannel;

use sdl2::pixels::Color;


use super::{GameObject, HitBox};


const BULLET_W: f64 = 8.0;
const BULLET_H: f64 = 4.0;
const BULLET_HALF_H: f64 = BULLET_H / 2.0;
const BULLET_SPEED: f64 = 240.0;


pub struct Bullet {
	location: (f64, f64),

	ballistics: Box<Ballistics>,
	lifetime: f64,
}

impl Bullet {

	pub fn hits(&self, body: &HitBox) -> bool {
		let hit_rect = self.ballistics.hit_rect(self);
		
		Rectangle::intersection(&hit_rect, body.frame())
		.map_or(false, |intersection| {

			AlphaChannel::intersect_box(body.collision_mask(), body.frame().x - body.bounds().x, body.frame().y - body.bounds().y, intersection)

		})
	}
}

impl GameObject<Bullet> for Bullet {

	fn location(&self) -> (f64, f64) {
		self.location
	}

	/// Render the bullet to the screen.
	fn render(&self, phi: &mut Phi) {
		// We will render this kind of bullet in bullet_color(time).
		//? This is exactly how we drew our first moving rectangle in the
		//? seventh part of this series.
		phi.renderer.set_draw_color(bullet_color(self.lifetime));
		phi.renderer.fill_rect(self.ballistics.hit_rect(self).to_sdl().unwrap()).unwrap();
	}

	fn update(mut self: Box<Bullet>, context: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
		self.lifetime += dt;

		if self.ballistics.update(&mut self, context, dt) {
			return Some(self);
		}
		None
	}		
}


#[derive(Clone, Copy)]
pub enum CannonType {
	RectBullet,
	SineBullet { amplitude: f64, angular_vel: f64 },
	DivergentBullet { a: f64, b: f64 },
}


pub fn spawn(cannon: CannonType, cannons_x: f64, cannon1_y: f64, cannon2_y: f64) -> Vec<Box<Bullet>> {
	let (ballistics_a, ballistics_b) = match cannon {
		CannonType::RectBullet => {
			let ballistics = Box::new(RectBulletBallistics {}) as Box<Ballistics>;

			(ballistics, ballistics)
		},
		CannonType::SineBullet { amplitude, angular_vel } => {
			let ballistics = Box::new(SineBulletBallistics {
				amplitude: amplitude,
				angular_vel: angular_vel,
			}) as Box<Ballistics>;

			(ballistics, ballistics)
		},
		CannonType::DivergentBullet { a, b } => {
			(
				Box::new(DivergentBulletBallistics {
					a: -a,
					b: b 
				}) as Box<Ballistics>, 
				Box::new(DivergentBulletBallistics {
					a: a,
					b: b 
				}) as Box<Ballistics>
			)
		},
	};

	vec![
	Box::new(Bullet {
		location: (cannons_x, cannon1_y - BULLET_HALF_H),

		ballistics: ballistics_a,
		lifetime: 0.0,
	}),
	Box::new(Bullet {
		location: (cannons_x, cannon2_y - BULLET_HALF_H),

		ballistics: ballistics_b,
		lifetime: 0.0,
	})]
}


trait Ballistics {

	fn hit_rect(&self, bullet: &Bullet) -> Rectangle {
		Rectangle {
			x: bullet.location.0,
			y: bullet.location.1,
			w: BULLET_W,
			h: BULLET_H
		}
	}

	fn update(&self, bullet: &mut Bullet, context: &Phi, dt: f64) -> bool {
		bullet.location.0 += BULLET_SPEED * dt;

		// If the bullet has left the screen, then delete it.
		bullet.location.0 < context.output_size().0		
	}
}


struct RectBulletBallistics;

impl Ballistics for RectBulletBallistics {
	/* Nothing to do */
}


struct SineBulletBallistics {
	//? Notice that the bounding box isn't stored directly. This means that
	//? we do not keep useless information. It also implies that we must compute
	//? the `sin` function every time we attempt to get the bounding box.
	amplitude: f64,
	angular_vel: f64,
}

impl Ballistics for SineBulletBallistics {

	fn hit_rect(&self, bullet: &Bullet) -> Rectangle {
		//? Just the general form of the sine function, minus the initial time.
		let dy = self.amplitude * f64::sin(self.angular_vel * bullet.lifetime);

		Rectangle {
			x: bullet.location.0,
			y: bullet.location.1 + dy,
			w: BULLET_W,
			h: BULLET_H
		}
	}
}


/// Bullet which follows a vertical trajectory given by:
///     a * ((t / b)^3 - (t / b)^2)
struct DivergentBulletBallistics {
	a: f64, // Influences the bump's height
	b: f64, // Influences the bump's width
}

impl Ballistics for DivergentBulletBallistics {

	fn hit_rect(&self, bullet: &Bullet) -> Rectangle {
		let dy = self.a *
		((bullet.lifetime / self.b).powi(3) -
			(bullet.lifetime / self.b).powi(2));

		Rectangle {
			x: bullet.location.0,
			y: bullet.location.1 + dy,
			w: BULLET_W,
			h: BULLET_H,
		}
	}

	fn update(&self, bullet: &mut Bullet, context: &Phi, dt: f64) -> bool {
		bullet.location.0 += BULLET_SPEED * dt;

		// If the bullet has left the screen, then delete it.
		let rect = self.hit_rect(bullet);
		let (w, h) = context.output_size();

		rect.x < w && rect.y < h && rect.y >= 0.0
	}	
}


fn bullet_color(t: f64) -> Color {
	let angle = (6.0 * t / ::std::f64::consts::PI) % 6.0;
	let x = (255.0 * (1.0 - (angle % 2.0 - 1.0).abs())) as u8;

	match angle as i32 {
		0 => Color::RGB(255, x, 0),
		1 => Color::RGB(x, 255, 0),
		2 => Color::RGB(0, 255, x),
		3 => Color::RGB(0, x, 255),
		4 => Color::RGB(x, 0, 255),
		5 => Color::RGB(255, 0, x),
		// This is impossible.
		_ => unreachable!()
	}
}