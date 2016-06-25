extern crate rand;

use phi::{Phi, View, ViewAction};
use phi::data::Rectangle;
use phi::gfx::{Renderable, AnimatedSprite, Sprite};

use sdl2::pixels::Color;

use views::background::Background;


// Constants
const SHIP_SPEED: f64 = 180.0;
const SHIP_W: f64 = 64.0;
const SHIP_H: f64 = 64.0;

const ASTEROIDS_WIDE: usize = 21;
const ASTEROIDS_HIGH: usize = 7;
const ASTEROIDS_TOTAL: usize = ASTEROIDS_WIDE * ASTEROIDS_HIGH - 4;
const ASTEROID_SIDE: f64 = 96.0;

const BULLET_SPEED: f64 = 240.0;
const BULLET_W: f64 = 8.0;
const BULLET_H: f64 = 4.0;


#[cfg(feature = "debugging")]
const DEBUG: bool = true;
#[cfg(not(feature = "debugging"))]
const DEBUG: bool= false;


/// The different states our ship might be in. In the image, they're ordered
/// from left to right, then from top to bottom.
#[derive(Clone, Copy)]
enum ShipFrame {
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


// Data types
struct Asteroid {
	sprite: AnimatedSprite,
	rect: Rectangle,
	vel: f64,
}

impl Asteroid {
	fn new(phi: &mut Phi) -> Asteroid {
		let mut asteroid =
		Asteroid {
			sprite: Asteroid::get_sprite(phi, 1.0),
			rect: Rectangle {
				w: 0.0,
				h: 0.0,
				x: 0.0,
				y: 0.0,
			},
			vel: 0.0,
		};

		asteroid.reset(phi);
		asteroid
	}

	fn reset(&mut self, phi: &mut Phi) {
		let (w, h) = phi.output_size();

		// FPS in [10.0, 30.0)
		//? `random<f64>()` returns a value between 0 and 1.
		//? `abs()` returns an absolute value
		self.sprite.set_fps(self::rand::random::<f64>().abs() * 20.0 + 10.0);

		// rect.y in the screen vertically
		self.rect = Rectangle {
			w: ASTEROID_SIDE,
			h: ASTEROID_SIDE,
			x: w,
			y: self::rand::random::<f64>().abs() * (h - ASTEROID_SIDE),
		};

		// vel in [50.0, 150.0)
		self.vel = self::rand::random::<f64>().abs() * 100.0 + 50.0;
	}

	fn get_sprite(phi: &mut Phi, fps: f64) -> AnimatedSprite {
		let asteroid_spritesheet = Sprite::load(&mut phi.renderer, "assets/asteroid.png").unwrap();
		let mut asteroid_sprites = Vec::with_capacity(ASTEROIDS_TOTAL);

		for yth in 0..ASTEROIDS_HIGH {
			for xth in 0..ASTEROIDS_WIDE {
				//? There are four asteroids missing at the end of the
				//? spritesheet: we do not want to render those.
				if ASTEROIDS_WIDE * yth + xth >= ASTEROIDS_TOTAL {
					break;
				}

				asteroid_sprites.push(
					asteroid_spritesheet.region(Rectangle {
						w: ASTEROID_SIDE,
						h: ASTEROID_SIDE,
						x: ASTEROID_SIDE * xth as f64,
						y: ASTEROID_SIDE * yth as f64,
					}).unwrap());
			}
		}
		AnimatedSprite::with_fps(asteroid_sprites, fps)
	}

	fn update(&mut self, phi: &mut Phi, dt: f64) {
		self.rect.x -= dt * self.vel;
		self.sprite.add_time(dt);

		if self.rect.x <= -ASTEROID_SIDE {
			self.reset(phi);
		}
	}

	fn render(&mut self, phi: &mut Phi) {
		self.sprite.render(&mut phi.renderer, self.rect);
	}
}


trait Bullet {
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


struct RectBullet {
	rect: Rectangle,
}

impl Bullet for RectBullet {
	fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
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
		phi.renderer.set_draw_color(Color::RGB(230, 230, 30));
		phi.renderer.fill_rect(self.rect.to_sdl().unwrap());
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
		phi.renderer.set_draw_color(Color::RGB(230, 230, 30));
		phi.renderer.fill_rect(self.rect().to_sdl().unwrap());
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
		phi.renderer.set_draw_color(Color::RGB(230, 230, 30));
		phi.renderer.fill_rect(self.rect().to_sdl().unwrap());
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


#[derive(Clone, Copy)]
enum CannonType {
	RectBullet,
	SineBullet { amplitude: f64, angular_vel: f64 },
	DivergentBullet { a: f64, b: f64 },
}

struct Ship {
	rect: Rectangle,
	sprites: Vec<Sprite>,
	current: ShipFrame,

	cannon: CannonType,
}

impl Ship {
	fn spawn_bullets(&self) -> Vec<Box<Bullet>> {
		let cannons_x = self.rect.x + 30.0;
		let cannon1_y = self.rect.y + 6.0;
		let cannon2_y = self.rect.y + SHIP_H - 10.0;

		match self.cannon {
			CannonType::RectBullet => {
				// One bullet at the tip of every cannon
				//? We could modify the initial position of the bullets by matching on
				//? `self.current : ShipFrame`, however there is not much point to this
				//? pedagogy-wise. You can try it out if you want. ;)
				vec![
				Box::new(RectBullet {
					rect: Rectangle {
						x: cannons_x,
						y: cannon1_y,
						w: BULLET_W,
						h: BULLET_H,
					}
				}),
				Box::new(RectBullet {
					rect: Rectangle {
						x: cannons_x,
						y: cannon2_y,
						w: BULLET_W,
						h: BULLET_H,
					}
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
}


pub struct ShipView {
	player: Ship,

	asteroid: Asteroid,
	bullets: Vec<Box<Bullet>>,

	bg_ambient: Background,
	bg_back: Background,
	bg_middle: Background,
	bg_front: Background,
}

impl ShipView {
	pub fn new (phi: &mut Phi) -> ShipView {
		let spritesheet = Sprite::load(&mut phi.renderer, "assets/spaceship2.png").unwrap();

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
		ShipView {
			player: Ship {
				rect: Rectangle {
					x: 64.0,
					y: 64.0,
					w: SHIP_W,
					h: SHIP_H,
				},
				sprites: sprites,
				current: ShipFrame::MidNorm,

				//? Let `RectBullet` be the default kind of bullet.
				cannon: CannonType::RectBullet,
			},
			asteroid: Asteroid::new(phi),
			//? We start with no bullets. Because the size of the vector will
			//? change drastically throughout the program, there is not much
			//? point in giving it a capacity.
			bullets: vec![],

			bg_ambient: Background::load(&phi.renderer, "assets/starAMB.png", 0.0).unwrap(),
			bg_back: Background::load(&phi.renderer, "assets/starBG.png", 20.0).unwrap(),
			bg_middle: Background::load(&phi.renderer, "assets/starMG.png", 40.0).unwrap(),
			bg_front: Background::load(&phi.renderer, "assets/starFG.png", 80.0).unwrap(),
		}
	}
}

impl View for ShipView {
	fn render(&mut self, phi: &mut Phi, elapsed: f64) -> ViewAction {
		if phi.events.now.quit {
			return ViewAction::Quit;
		}
		if phi.events.now.key_escape == Some(true) {
			return ViewAction::ChangeView(Box::new(::views::menu_main::MainMenuView::new(phi)));
		}
		// Change the player's cannons

		if phi.events.now.key_1 == Some(true) {
			self.player.cannon = CannonType::RectBullet;
		}
		if phi.events.now.key_2 == Some(true) {
			self.player.cannon = CannonType::SineBullet {
				amplitude: 10.0,
				angular_vel: 15.0,
			};
		}
		if phi.events.now.key_3 == Some(true) {
			self.player.cannon = CannonType::DivergentBullet {
				a: 100.0,
				b: 1.2,
			};
		}
		let diagonal = 
		(phi.events.key_up ^ phi.events.key_down) && 
		(phi.events.key_left ^ phi.events.key_right);

		let moved =
		if diagonal { 1.0 / 2.0f64.sqrt() }
		else { 1.0 } * SHIP_SPEED * elapsed;

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

		self.player.rect.x += dx;
		self.player.rect.y += dy;

		// The movable region spans the entire height of the window and 70% of its
		// width. This way, the player cannot get to the far right of the screen, where
		// we will spawn the asteroids, and get immediately eliminated.
		//
		// We restrain the width because most screens are wider than they are high.
		let movable_region = Rectangle {
			x: 0.0,
			y: 0.0,
			w: phi.output_size().0 * 0.70,
			h: phi.output_size().1,
		};

		self.player.rect = self.player.rect.move_inside(movable_region).unwrap();

		// Select the appropriate sprite of the ship to show.
		self.player.current =
		if dx == 0.0 && dy < 0.0       { ShipFrame::UpNorm }
		else if dx > 0.0 && dy < 0.0   { ShipFrame::UpFast }
		else if dx < 0.0 && dy < 0.0   { ShipFrame::UpSlow }
		else if dx == 0.0 && dy == 0.0 { ShipFrame::MidNorm }
		else if dx > 0.0 && dy == 0.0  { ShipFrame::MidFast }
		else if dx < 0.0 && dy == 0.0  { ShipFrame::MidSlow }
		else if dx == 0.0 && dy > 0.0  { ShipFrame::DownNorm }
		else if dx > 0.0 && dy > 0.0   { ShipFrame::DownFast }
		else if dx < 0.0 && dy > 0.0   { ShipFrame::DownSlow }
		else { unreachable!() };

		// Update bullets
		//? Set `self.bullets` to be the empty vector, and put its content inside of
		//? `old_bullets`, which we can move without borrow-checker issues.
		let old_bullets = ::std::mem::replace(&mut self.bullets, vec![]);

		//? Upon assignment, the old value of `self.bullets`, namely the empty vector,
		//? will be freed automatically, because its owner no longer refers to it.
		//? We can then update the bullet quite simply.
		self.bullets =
		old_bullets.into_iter()
		.filter_map(|bullet| bullet.update(phi, elapsed))
		.collect();

		// Update the asteroid
		self.asteroid.update(phi, elapsed);

		// Allow the player to shoot after the bullets are updated, so that,
		// when rendered for the first time, they are drawn wherever they
		// spawned.
		//
		//? In this case, we ensure that the new bullets are drawn at the tips
		//? of the cannons.
		//?
		//? The `Vec::append` method moves the content of `spawn_bullets` at
		//? the end of `self.bullets`. After this is done, the vector returned
		//? by `spawn_bullets` will be empty.
		if phi.events.now.key_space == Some(true) {
			self.bullets.append(&mut self.player.spawn_bullets());
		}

		// Clear the screen
		phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
		phi.renderer.clear();

		// Render Backgrounds
		self.bg_ambient.render(&mut phi.renderer, elapsed);
		self.bg_back.render(&mut phi.renderer, elapsed);
		self.bg_middle.render(&mut phi.renderer, elapsed);

		// Render the bounding box (for debugging purposes)
		if DEBUG {
			phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
			phi.renderer.fill_rect(self.player.rect.to_sdl().unwrap()).unwrap();
		}
		self.player.sprites[self.player.current as usize]
		.render(&mut phi.renderer, self.player.rect);

		// Render the asteroid
		self.asteroid.render(phi);
		// Render the bullets
		for bullet in &self.bullets {
			bullet.render(phi);
		}
		// Render the foreground
		self.bg_front.render(&mut phi.renderer, elapsed);

		ViewAction::None
	}
}