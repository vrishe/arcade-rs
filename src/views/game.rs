extern crate rand;

use phi::{Phi, View, ViewAction};
use phi::data::{Rectangle, MaybeAlive};
use phi::gfx::{AlphaChannel, AnimatedSprite, AnimatedSpriteDescr, Collide, BoxCollide, Renderable, Sprite};

use sdl2::pixels::Color;
use sdl2::rwops::RWops;
use sdl2_image::ImageRWops;
use sdl2_mixer::Chunk;

use std::path::Path;
use std::rc::Rc;

use views::background::Background;
use views::bullets::{Bullet, CannonType};


// Constants
const ASTEROIDS_WIDE: usize = 21;
const ASTEROIDS_HIGH: usize = 7;
const ASTEROIDS_TOTAL: usize = ASTEROIDS_WIDE * ASTEROIDS_HIGH - 4;
const ASTEROID_SIDE: f64 = 96.0;

const EXPLOSIONS_WIDE: usize = 5;
const EXPLOSIONS_HIGH: usize = 4;
const EXPLOSIONS_TOTAL: usize = 17;
const EXPLOSION_SIDE: f64 = 96.0;
const EXPLOSION_FPS: f64 = 16.0;
const EXPLOSION_DURATION: f64 = 1.0 / EXPLOSION_FPS * EXPLOSIONS_TOTAL as f64;

const PLAYER_SPEED: f64 = 180.0;
const PLAYER_W: f64 = 64.0;
const PLAYER_H: f64 = 64.0;

const SHOT_DELAY: f64 = 1.0 / 7.62;


#[cfg(feature="debug")]
const DEBUG: bool = true;
#[cfg(not(feature="debug"))]
const DEBUG: bool= false;


/// The different states our Player might be in. In the image, they're ordered
/// from left to right, then from top to bottom.
#[derive(Clone, Copy)]
enum PlayerFrame {
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


trait GameBody {

	fn rect(&self) -> &Rectangle; 

	fn frame(&self) -> Rectangle;

	fn alpha(&self) -> &AlphaChannel; 

	fn collide(&self, another: &GameBody) -> bool {
		Rectangle::intersection(self.rect(), another.rect())
		.map_or(false, |intersection| {
			let (self_frame, another_frame) = (self.frame(), another.frame());

			<AlphaChannel as Collide>::collide(self.alpha(), self.rect().x - self_frame.x, self.rect().y - self_frame.y, 
				another.alpha(), another.rect().x - another_frame.x, another.rect().y - another_frame.y, intersection)
		})
	}

	fn overlap(&self, area: &Rectangle) -> bool {
		Rectangle::intersection(self.rect(), area)
		.map_or(false, |intersection| {
			let self_frame = self.frame();

			<AlphaChannel as BoxCollide>::collide(self.alpha(), self.rect().x - self_frame.x, self.rect().y - self_frame.y, intersection)
		})
	}
}


// Data types
struct Asteroid {
	rect: Rectangle,
	sprite: AnimatedSprite,

	alpha: AlphaChannel,

	vel: f64,
}

impl Asteroid {
	fn factory(phi: &mut Phi) -> AsteroidFactory {
		let surface_reader = RWops::from_file("assets/asteroid.png", "rb").unwrap();
		let surface = surface_reader.load().unwrap();

		let alpha = unsafe { AlphaChannel::from_surface(&surface, Some(0.5)).unwrap() };
		let spritesheet = Sprite::from_surface(&phi.renderer, &surface).unwrap();

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

	fn update(mut self, dt: f64) -> Option<Asteroid> {
		self.rect.x -= dt * self.vel;
		self.sprite.add_time(dt);

		if self.rect.x > -ASTEROID_SIDE {
			return Some(self)
		}
		None
	}

	fn render(&self, phi: &mut Phi) {
		if DEBUG {
			// Render the bounding box
			phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
			phi.renderer.fill_rect(self.rect.to_sdl().unwrap()).unwrap();
		}
		self.sprite.render(&mut phi.renderer, self.rect);
	}
}

impl<'a> GameBody for Asteroid {
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


struct AsteroidFactory {
	alpha: Rc<AlphaChannel>,
	sprite: AnimatedSprite,
}

impl AsteroidFactory {
	fn random(&self, phi: &mut Phi) -> Asteroid {
		let (w, h) = phi.output_size();

		// FPS in [10.0, 30.0)
		let mut sprite = self.sprite.clone();

		sprite.set_fps(self::rand::random::<f64>().abs() * 20.0 + 10.0);

		Asteroid {
			// In the screen vertically, and over the right of the screen
			// horizontally.
			rect: Rectangle {
				w: ASTEROID_SIDE,
				h: ASTEROID_SIDE,
				x: w,
				y: self::rand::random::<f64>().abs() * (h - ASTEROID_SIDE),
			},
			sprite: sprite,

			alpha: self.alpha.as_ref().clone(),

			// vel in [50.0, 150.0)
			vel: self::rand::random::<f64>().abs() * 100.0 + 50.0,
		}
	}
}


struct Explosion {
	rect: Rectangle,
	sprite: AnimatedSprite,

	//? Keep how long its been arrived, so that we destroy the explosion once
	//? its animation is finished.
	alive_since: f64,
}

impl Explosion {
	fn factory(phi: &mut Phi) -> ExplosionFactory {
		ExplosionFactory {
			sprite: AnimatedSprite::load_with_fps(
				"assets/explosion.png", 
				phi, AnimatedSpriteDescr {
					total_frames: EXPLOSIONS_TOTAL,
					frames_high: EXPLOSIONS_HIGH,
					frames_wide: EXPLOSIONS_WIDE,
					frame_w: EXPLOSION_SIDE,
					frame_h: EXPLOSION_SIDE,
				}, 
				EXPLOSION_FPS),
		}
	}

	fn update(mut self, dt: f64) -> Option<Explosion> {
		self.alive_since += dt;
		self.sprite.add_time(dt);

		if self.alive_since < EXPLOSION_DURATION {
			return Some(self);
		}
		None
	}

	fn render(&self, phi: &mut Phi) {
		self.sprite.render(&mut phi.renderer, self.rect);
	}
}


struct ExplosionFactory {
	sprite: AnimatedSprite,
}

impl ExplosionFactory {
	fn at_center(&self, center: (f64, f64)) -> Explosion {
		// FPS in [10.0, 30.0)
		let sprite = self.sprite.clone();

		Explosion {
			sprite: sprite,

			// In the screen vertically, and over the right of the screen
			// horizontally.
			rect: Rectangle::with_size(EXPLOSION_SIDE, EXPLOSION_SIDE)
			.center_at(center),

			alive_since: 0.0,
		}
	}
}


struct Player {
	rect: Rectangle,
	sprites: Vec<Sprite>,

	alpha: AlphaChannel,

	cannon: CannonType,
	current: PlayerFrame,
}

impl Player {
	pub fn new(phi: &mut Phi) -> Player {
		let surface_reader = RWops::from_file("assets/spaceship2.png", "rb").unwrap();
		let surface = surface_reader.load().unwrap();

		let alpha = unsafe { AlphaChannel::from_surface(&surface, Some(0.25)).unwrap() };
		let spritesheet = Sprite::from_surface(&phi.renderer, &surface).unwrap();
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
		println!("{:?}", alpha);

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

		::views::bullets::spawn_bullets(self.cannon, cannons_x, cannon1_y, cannon2_y)
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
		if DEBUG {
			phi.renderer.set_draw_color(Color::RGB(10, 200, 50));
			phi.renderer.fill_rect(self.rect.to_sdl().unwrap()).unwrap();
		}

		// Render the ship's current sprite.
		self.sprites[self.current as usize]
		.render(&mut phi.renderer, self.rect);
	}
}

impl GameBody for Player {
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


pub struct GameView {
	player: Player,
	shot_time: f64,

	asteroid_factory: AsteroidFactory,
	explosion_factory: ExplosionFactory,

	asteroids: Vec<Asteroid>,
	bullets: Vec<Box<Bullet>>,
	explosions: Vec<Explosion>,

	bg_ambient: Background,
	bg_back: Background,
	bg_middle: Background,
	bg_front: Background,

	bullet_sound: Chunk,
	explosion_sound: Chunk,
}

impl GameView {
	pub fn new (phi: &mut Phi) -> GameView {

		GameView {
			player: Player::new(phi),
			shot_time: SHOT_DELAY,

			asteroid_factory: Asteroid::factory(phi),
			explosion_factory: Explosion::factory(phi),

			asteroids: vec![],
			bullets: vec![],
			explosions: vec![],

			bg_ambient: Background::load(&phi.renderer, "assets/starAMB.png", 0.0).unwrap(),
			bg_back: Background::load(&phi.renderer, "assets/starBG.png", 20.0).unwrap(),
			bg_middle: Background::load(&phi.renderer, "assets/starMG.png", 40.0).unwrap(),
			bg_front: Background::load(&phi.renderer, "assets/starFG.png", 80.0).unwrap(),

			bullet_sound: Chunk::from_file(Path::new("assets/bullet.ogg")).unwrap(),
			explosion_sound: Chunk::from_file(Path::new("assets/explosion.ogg")).unwrap()
		}
	}
}

impl View for GameView {
	fn update(mut self: Box<Self>, phi: &mut Phi, elapsed: f64) -> ViewAction {
		if phi.events.now.quit {
			return ViewAction::Quit;
		}
		if phi.events.now.key_escape == Some(true) {
			return ViewAction::Render(Box::new(::views::menu_main::MainMenuView::new(phi)));
		}
		// This is a tricky 'game' update block, as we have troubles
		// with the way, how Rust handles runtime safety for references.
		{
			let game = &mut *self;

			//? Upon assignment, the old value of `self.bullets`, namely the empty vector,
			//? will be freed automatically, because its owner no longer refers to it.
			//? We can then update the bullet quite simply.
			game.bullets = ::std::mem::replace(&mut game.bullets, vec![]).into_iter()
			.filter_map(|bullet| bullet.update(phi, elapsed))
			.collect();

			// Update the asteroids
			game.asteroids = ::std::mem::replace(&mut game.asteroids, vec![]).into_iter()
			.filter_map(|asteroid| asteroid.update(elapsed))
			.collect();

			// Update the explosions
			game.explosions = ::std::mem::replace(&mut game.explosions, vec![]).into_iter()
			.filter_map(|explosion| explosion.update(elapsed))
			.collect();

			//? We keep track of whether or not the player is alive.
			let mut player_alive = true;

			//? First, go through the bullets and wrap them in a `MaybeAlive`, so that we
			//? can keep track of which got into a collision and which did not.
			let mut transition_bullets = ::std::mem::replace(&mut game.bullets, vec![]).into_iter()
			.map(|bullet| MaybeAlive { alive: true, value: bullet })
			.collect::<Vec<_>>();

			game.asteroids = ::std::mem::replace(&mut game.asteroids, vec![]).into_iter()
			.filter_map(|asteroid| {
				// By default, the asteroid has not been in a collision.
				let mut asteroid_alive = true;

				for bullet in &mut transition_bullets {
					//? Notice that we refer to the bullet as `bullet.value`
					//? because it has been wrapped in `MaybeAlive`.
					if asteroid.overlap(&bullet.value.rect()) {
						asteroid_alive = false;
						//? We go through every bullet and "kill" those that collide
						//? with the asteroid. We do this for every asteroid.
						bullet.alive = false;
					}
				}
				// The player's Player is destroyed if it is hit by an asteroid.
				// In which case, the asteroid is also destroyed.
				if asteroid.collide(&game.player) {
					asteroid_alive = false;
					player_alive = false;
				}
				//? Then, we use the magic of `filter_map` to keep only the asteroids
				//? that didn't explode.
				if asteroid_alive {
					return Some(asteroid)
				}
				game.explosions.push(
					game.explosion_factory.at_center(
						asteroid.rect.center()));

				phi.play_sound(&game.explosion_sound);

				None
			})
			.collect();

			//? Finally, we use once again the magic of `filter_map` to keep only the
			//? bullets that are still alive.
			game.bullets = transition_bullets.into_iter()
			.filter_map(MaybeAlive::as_option)
			.collect();

			// TODO
			// For the moment, we won't do anything about the player dying. This will be
			// the subject of a future episode.
			if !player_alive {
				println!("The player's Ship has been destroyed.");
			}
			game.player.update(phi, elapsed);

			// Allow the player to shoot after the bullets are updated, so that,
			// when rendered for the first time, they are drawn wherever they
			// spawned.
			//
			//? In this case, we ensure that the new bullets are drawn at the tips
			//? of the cannons.
			//?
			//? The `Vec::append` method moves the content of `spawn_bullets` at
			//? the end of `game.bullets`. After this is done, the vector returned
			//? by `spawn_bullets` will be empty.
			if phi.events.key_space {
				let mut shots_fired = (game.shot_time / SHOT_DELAY) as isize;

				if shots_fired > 0 {
					game.shot_time = 0.0;

					while shots_fired > 0 {
						game.bullets.append(&mut game.player.spawn_bullets());

						phi.play_sound(&game.bullet_sound);
						shots_fired -= 1;
					}					
				} else {
					game.shot_time += elapsed;
				}				
			} else {
				game.shot_time = SHOT_DELAY;
			}
			// Randomly create an asteroid about once every 100 frames, that is,
			// a bit more often than once every two seconds.
			if self::rand::random::<usize>() % 100 == 0 {
				game.asteroids.push(game.asteroid_factory.random(phi));
			}
			game.bg_ambient.update(elapsed);
			game.bg_back.update(elapsed);
			game.bg_middle.update(elapsed);
			game.bg_front.update(elapsed);
		}
		ViewAction::Render(self)
	}

	fn render(&self, phi: &mut Phi) {
		// Clear the screen
		phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
		phi.renderer.clear();

		// Render Backgrounds
		self.bg_ambient.render(&mut phi.renderer);
		self.bg_back.render(&mut phi.renderer);
		self.bg_middle.render(&mut phi.renderer);

		// Render asteroids
		for asteroid in &self.asteroids {
			asteroid.render(phi);
		}
		self.player.render(phi);

		// Render bullets
		for bullet in &self.bullets {
			bullet.render(phi);
		}
		// Render explosions
		for explosion in &self.explosions {
			explosion.render(phi);
		}
		// Render the foreground
		self.bg_front.render(&mut phi.renderer);
	}
}