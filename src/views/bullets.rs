use phi::Phi;
use phi::data::Rectangle;

use sdl2::pixels::Color;


#[derive(Clone, Copy)]
pub enum CannonType {
	RectBullet,
	SineBullet { amplitude: f64, angular_vel: f64 },
	DivergentBullet { a: f64, b: f64 },
}


pub trait Bullet {
	/// Update the bullet.
	/// If the bullet should be destroyed, e.g. because it has left the screen,
	/// then return `None`.
	/// Otherwise, return `Some(update_bullet)`.
	//?
	//? Notice how we use `Box<Self>` as the type of `self`. This means: keep
	//? this data behind a pointer, but `move` the pointer. You should note that
	//? *we are not copying the value*: we are only copying the _address_ at
	//? which the value is stored in memory, which has a negligible cost. We can
	//? do this because Rust will automatically free the memory once the `Box`
	//? that contains it is itself destroyed.
	fn update(self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>>;

	/// Render the bullet to the screen.
	//? Here, we take an immutable reference to the bullet, because we do not
	//? need to change its value to draw it. This is the same as before.
	fn render(&self, phi: &mut Phi);

	/// Return the bullet's bounding box.
	//? This is also the same as before.
	fn rect(&self) -> Rectangle;
}


const BULLET_W: f64 = 8.0;
const BULLET_H: f64 = 4.0;

pub fn spawn_bullets(cannon: CannonType, cannons_x: f64, cannon1_y: f64, cannon2_y: f64) -> Vec<Box<Bullet>> {
	match cannon {
		CannonType::RectBullet => {
			// One bullet at the tip of every cannon
			//? We could modify the initial position of the bullets by matching on
			//? `self.current : PlayerFrame`, however there is not much point to this
			//? pedagogy-wise. You can try it out if you want. ;)
			vec![
			Box::new(RectBullet {
				rect: Rectangle {
					x: cannons_x,
					y: cannon1_y,
					w: BULLET_W,
					h: BULLET_H,
				},
				total_time: 0.0,
			}),
			Box::new(RectBullet {
				rect: Rectangle {
					x: cannons_x,
					y: cannon2_y,
					w: BULLET_W,
					h: BULLET_H,
				},
				total_time: 0.0,
			}),
			]
		},
		CannonType::SineBullet { amplitude, angular_vel } => {
			vec![
			Box::new(SineBullet {
				pos_x: cannons_x,
				origin_y: cannon1_y,
				amplitude: amplitude,
				angular_vel: angular_vel,
				total_time: 0.0,
			}),
			Box::new(SineBullet {
				pos_x: cannons_x,
				origin_y: cannon2_y,
				amplitude: amplitude,
				angular_vel: angular_vel,
				total_time: 0.0,
			}),
			]
		},
		CannonType::DivergentBullet { a, b } => {
			vec![
			// If a,b > 0, eventually goes upwards
			Box::new(DivergentBullet {
				pos_x: cannons_x,
				origin_y: cannon1_y,
				a: -a,
				b: b,
				total_time: 0.0,
			}),
			// If a,b > 0, eventually goes downwards
			Box::new(DivergentBullet {
				pos_x: cannons_x,
				origin_y: cannon2_y,
				a: a,
				b: b,
				total_time: 0.0,
			}),
			]
		},
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


const BULLET_SPEED: f64 = 240.0;

struct RectBullet {
	rect: Rectangle,
	total_time: f64,
}

impl Bullet for RectBullet {
	fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
		self.total_time += dt;
		self.rect.x += BULLET_SPEED * dt;

		let (w, _) = phi.output_size();
		// If the bullet has left the screen, then delete it.
		if self.rect.x < w {
			return Some(self)
		}
		None
	}

	/// Render the bullet to the screen.
	fn render(&self, phi: &mut Phi) {
		// We will render this kind of bullet in yellow.
		//? This is exactly how we drew our first moving rectangle in the
		//? seventh part of this series.
		phi.renderer.set_draw_color(bullet_color(self.total_time));
		phi.renderer.fill_rect(self.rect.to_sdl().unwrap()).unwrap();
	}

	/// Return the bullet's bounding box.
	fn rect(&self) -> Rectangle {
		self.rect
	}
}


struct SineBullet {
	//? Notice that the bounding box isn't stored directly. This means that
	//? we do not keep useless information. It also implies that we must compute
	//? the `sin` function every time we attempt to get the bounding box.
	pos_x: f64,
	origin_y: f64,
	amplitude: f64,
	angular_vel: f64,
	total_time: f64,
}

impl Bullet for SineBullet {
	fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
		//? We store the total time...
		self.total_time += dt;
		self.pos_x += BULLET_SPEED * dt;

		let (w, _) = phi.output_size();
		// If the bullet has left the screen, then delete it.
		if self.rect().x < w {
			return Some(self)
		}
		None
	}

	/// Render the bullet to the screen.
	fn render(&self, phi: &mut Phi) {
		// We will render this kind of bullet in yellow.
		//? This is exactly how we drew our first moving rectangle in the
		//? seventh part of this series.
		phi.renderer.set_draw_color(bullet_color(self.total_time));
		phi.renderer.fill_rect(self.rect().to_sdl().unwrap()).unwrap();
	}

	/// Return the bullet's bounding box.
	fn rect(&self) -> Rectangle {
		//? Just the general form of the sine function, minus the initial time.
		let dy = self.amplitude * f64::sin(self.angular_vel * self.total_time);
		Rectangle {
			x: self.pos_x,
			y: self.origin_y + dy,
			w: BULLET_W,
			h: BULLET_H,
		}
	}
}


/// Bullet which follows a vertical trajectory given by:
///     a * ((t / b)^3 - (t / b)^2)
struct DivergentBullet {
	pos_x: f64,
	origin_y: f64,
	a: f64, // Influences the bump's height
	b: f64, // Influences the bump's width
	total_time: f64,
}

impl Bullet for DivergentBullet {
	fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
		self.total_time += dt;
		self.pos_x += BULLET_SPEED * dt;

		// If the bullet has left the screen, then delete it.
		let (w, h) = phi.output_size();
		let rect = self.rect();

		if rect.x < w && rect.x >= 0.0 &&
		rect.y < h && rect.y >= 0.0 {
			return Some(self)
		}
		None
	}

	fn render(&self, phi: &mut Phi) {
		// We will render this kind of bullet in yellow.
		phi.renderer.set_draw_color(bullet_color(self.total_time));
		phi.renderer.fill_rect(self.rect().to_sdl().unwrap()).unwrap();
	}

	fn rect(&self) -> Rectangle {
		let dy = self.a *
		((self.total_time / self.b).powi(3) -
			(self.total_time / self.b).powi(2));

		Rectangle {
			x: self.pos_x,
			y: self.origin_y + dy,
			w: BULLET_W,
			h: BULLET_H,
		}
	}
}