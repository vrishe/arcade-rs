extern crate rand;


mod asteroid;
mod blast;
mod bullet;
mod explosion;
mod player;


use phi::{Phi, View, ViewAction};
use phi::data::Rectangle;
use phi::gfx::{AlphaChannel, Renderable, Sprite};

use sdl2::pixels::Color;
use sdl2::rwops::RWops;
use sdl2::image::ImageRWops;
use sdl2::mixer::Chunk;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use hud::background::Background;
use hud::button::Button;


use self::asteroid::{Asteroid, AsteroidFactory};
use self::blast::Blast;
use self::bullet::Bullet;
use self::explosion::{Explosion, ExplosionFactory};
use self::player::Player;


const SHOT_DELAY: f64 = 1.0 / 7.62;


pub struct GameView {
	player: Rc<RefCell<Box<Player>>>,
	shot_time: f64,

	asteroid_factory: AsteroidFactory,
	explosion_factory: ExplosionFactory,

	asteroids: Vec<Box<Asteroid>>,
	blasts: Vec<Box<Blast>>,
	bullets: Vec<Box<Bullet>>,
	explosions: Vec<Box<Explosion>>,

	bg_ambient: Background,
	bg_back: Background,
	bg_middle: Background,
	bg_front: Background,

	buttons_ammo: Vec<GameButton>,

	bullet_sound: Chunk,
	explosion_sound: Chunk,
}

impl GameView {
	pub fn new (phi: &mut Phi) -> GameView {
		let mut buttons_ammo = Vec::with_capacity(3);

		buttons_ammo.push(GameButton::new(phi, "assets/sprites/button_ammo0.png", "1", (32.0, 32.0), (1.5, 1.5, 3.5, 1.5)));
		buttons_ammo.push(GameButton::new(phi, "assets/sprites/button_ammo1.png", "2", (32.0, 32.0), (1.5, 1.5, 3.5, 1.5)));
		buttons_ammo.push(GameButton::new(phi, "assets/sprites/button_ammo2.png", "3", (32.0, 32.0), (1.5, 1.5, 3.5, 1.5)));

		GameView {
			player: Rc::new(RefCell::new(Box::new(Player::new(phi)))),
			shot_time: SHOT_DELAY,

			asteroid_factory: Asteroid::factory(phi),
			explosion_factory: Explosion::factory(phi),

			asteroids: vec![],
			blasts: vec![],
			bullets: vec![],
			explosions: vec![],

			bg_ambient: Background::load(&phi.renderer, "assets/backgrounds/starAMB.png", 0.0).unwrap(),
			bg_back: Background::load(&phi.renderer, "assets/backgrounds/starBG.png", 20.0).unwrap(),
			bg_middle: Background::load(&phi.renderer, "assets/backgrounds/starMG.png", 40.0).unwrap(),
			bg_front: Background::load(&phi.renderer, "assets/backgrounds/starFG.png", 80.0).unwrap(),

			buttons_ammo: buttons_ammo,

			bullet_sound: Chunk::from_file(Path::new("assets/sounds/bullet.ogg")).unwrap(),
			explosion_sound: Chunk::from_file(Path::new("assets/sounds/explosion.ogg")).unwrap()
		}
	}
}

macro_rules! explode (
	( $game_ident: ident : $context_ident: ident @ $center_expr: expr ) => { 
		{
			$game_ident.explosions.push(Box::new(
				$game_ident.explosion_factory.at_center($center_expr)));

			$context_ident.play_sound(&$game_ident.explosion_sound);
		}
	}
);

impl View for GameView {
	fn update(mut self: Box<Self>, context: &mut Phi, elapsed: f64) -> ViewAction {
		if context.events.now.quit {
			return ViewAction::Quit;
		}
		if context.events.now.key_escape == Some(true) {
			return ViewAction::Render(Box::new(::views::menu_main::MainMenuView::new(context)));
		}
		// This is a tricky 'game' update block, as we have troubles
		// with the way, how Rust handles runtime safety for references.
		{
			let game = &mut *self;

			let asteroids_left: Vec<Box<Asteroid>> = ::std::mem::replace(&mut game.asteroids, vec![]).into_iter()
			.filter_map(|asteroid| {
				// By default, the asteroid has not been in a collision.
				let mut hits_count = 0;

				for bullet in &mut game.bullets {
					//? Notice that we refer to the bullet as `bullet.value`
					//? because it has been wrapped in `MaybeAlive`.
					match bullet.hits_at(&*asteroid) {
						Some(hit_location) => {
							game.blasts.push(Box::new(Blast::new(hit_location)));
							explode!(game:context @ hit_location);

							hits_count += 1;
						},
						_ => {}
					}
				}
				let mut player = game.player.borrow_mut();
				//? Then, we use the magic of `filter_map` to keep only the asteroids
				//? that didn't explode.
				if hits_count == 0 {
					// The player's Player is destroyed if it is hit by an asteroid.
					// In which case, the asteroid is also destroyed.
					if !player.is_alive() || !player.is_hit_by(&*asteroid) {
						return asteroid.update(context, elapsed);
					}
					explode!(game:context @ player.frame().center());
				}
				None
			})
			.collect();

			game.blasts = ::std::mem::replace(&mut game.blasts, vec![]).into_iter()
			.filter_map(|blast| { blast.update(context, elapsed) })
			.collect();

			game.asteroids = asteroids_left.into_iter()
			.filter_map(|asteroid| {
				let tl = asteroid.frame().location();
				let tr = (tl.0 + asteroid.frame().w, tl.1);
				let br = (tr.0, tl.1 + asteroid.frame().h);
				let bl = (tl.0, br.1);

				for blast in &mut game.blasts {
					if blast.hits_at(tl) || blast.hits_at(br)
					|| blast.hits_at(tr) || blast.hits_at(bl) {

						explode!(game:context @ asteroid.frame().center());

						return None;
					}
				}
				Some(asteroid)
			})
			.collect();

			game.bullets = ::std::mem::replace(&mut game.bullets, vec![]).into_iter()
			.filter_map(|bullet| {
				let center = bullet.center();

				for blast in &mut game.blasts {
					if blast.hits_at(center) {
						return None;
					}
				}
				bullet.update(context, elapsed)
			})
			.collect();

			game.explosions = ::std::mem::replace(&mut game.explosions, vec![]).into_iter()
			.filter_map(|explosion| { explosion.update(context, elapsed) })
			.collect();

			let mut player = game.player.borrow_mut();

			if player.is_alive() {
				player.update_ref(context, elapsed);
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
				if context.events.key_space {
					let mut shots_fired = (game.shot_time / SHOT_DELAY) as isize;

					if shots_fired > 0 {
						game.shot_time = 0.0;

						while shots_fired > 0 {
							game.bullets.append(&mut player.shoot());

							context.play_sound(&game.bullet_sound);
							shots_fired -= 1;
						}					
					} else {
						game.shot_time += elapsed;
					}				
				} else {
					game.shot_time = SHOT_DELAY;
				}
				let output_size = context.output_size();

				let mut opaque = true;
				let mut changed = false;
				for i in 0..game.buttons_ammo.len() {
					let button = &mut game.buttons_ammo[i];
					let button_frame = *button.frame();

					let is_selected = player.get_ammo() == i;
					let vertical_offset = if is_selected { 10.5 } else { 8.0 };

					button.set_location(8.0 + i as f64 * (button_frame.w + 2.0), output_size.1 - vertical_offset - button_frame.h);
					changed |= button.set_state(is_selected as usize);

					if opaque && button.frame().overlaps(player.frame()) {
						opaque = false;
					}
				}
				let alpha_delta = if opaque { 4.0 * elapsed } else { -4.0 * elapsed };

				for button in &mut game.buttons_ammo {
					let alpha = if !changed { button.get_alpha() } else { 1.0 };

					button.set_alpha(alpha + alpha_delta);
				}						
			} else {
				// TODO
				// For the moment, we won't do anything about the player dying. This will be
				// the subject of a future episode.

				println!("The player's Ship has been destroyed.");
			}
			// Randomly create an asteroid about once every 100 frames, that is,
			// a bit more often than once every two seconds.
			if self::rand::random::<usize>() % 100 == 0 {
				game.asteroids.push(Box::new(game.asteroid_factory.random(context)));
			}
			game.bg_ambient.update(elapsed);
			game.bg_back.update(elapsed);
			game.bg_middle.update(elapsed);
			game.bg_front.update(elapsed);
		}
		ViewAction::Render(self)
	}

	fn render(&self, context: &mut Phi) {
		// Clear the screen
		context.renderer.set_draw_color(Color::RGB(0, 0, 0));
		context.renderer.clear();

		// Render Backgrounds
		self.bg_ambient.render(&mut context.renderer);
		self.bg_back.render(&mut context.renderer);
		self.bg_middle.render(&mut context.renderer);

		// Render asteroids
		for asteroid in &self.asteroids {
			asteroid.render(context);
		}
		let player = self.player.borrow();

		if player.is_alive() {
			player.render(context);
		}
		// Render bullets
		for bullet in &self.bullets {
			bullet.render(context);
		}
		// Render blasts
		for blast in &self.blasts {
			blast.render(context);
		}
		// Render explosions
		for explosion in &self.explosions {
			explosion.render(context);
		}
		// Render the foreground
		self.bg_front.render(&mut context.renderer);

		// Render HUD
		if self.player.borrow().is_alive() {
			for button in &self.buttons_ammo {
				button.render(context);
			}			
		}
	}
}


struct GameButton {
	button: Button,
	label: Sprite,
	label_frame: Rectangle,
}

impl GameButton {

	pub fn new (context: &mut Phi, path: &str, label: &str, size: (f64, f64), padding: (f64, f64, f64, f64)) -> GameButton {
		let label_sprite = context.ttf_str_sprite(label, "assets/fonts/BlackOpsOne-Regular.ttf", (size.1 / 3.36) as u16, Color::RGB(255, 255, 255)).unwrap();
		let scale = ((size.0 - padding.0 - padding.2) / label_sprite.size().0).min((size.1 - padding.1 - padding.3) / label_sprite.size().1).min(1.0);
		let label_size = (label_sprite.size().0 * scale, label_sprite.size().1 * scale);

		GameButton {
			button: Button::load(&context.renderer, path, size).unwrap(),
			label: label_sprite,
			label_frame: Rectangle {
				x: size.0 - label_size.0 - padding.2,
				y: size.1 - label_size.1 - padding.3,
				w: label_size.0,
				h: label_size.1
			}
		}
	}


	pub fn set_location(&mut self, x: f64, y:f64) {
		self.button.set_location(x, y);
	}

	pub fn get_alpha(&self) -> f64 {
		self.label.get_alpha()
	}

	pub fn set_alpha(&mut self, alpha: f64) {
		self.button.set_alpha(alpha);
		self.label.set_alpha(alpha);
	}

	pub fn set_state(&mut self, state: usize) -> bool {
		if self.button.get_state() != state {
			self.button.set_state(state);

			return true;
		}
		false
	}


	pub fn frame(&self) -> &Rectangle {
		self.button.frame()
	}


	pub fn render(&self, context: &mut Phi) {
		self.button.render(&mut context.renderer);
		self.label.render(&mut context.renderer, Rectangle {
			x: self.button.frame().x + self.label_frame.x,
			y: self.button.frame().y + self.label_frame.y,
			..self.label_frame
		});
	}
}


trait GameObject<T> {

	fn is_alive(&self) -> bool;

	fn location(&self) -> (f64, f64);


	fn update(self: Box<Self>, context: &mut Phi, dt: f64) -> Option<Box<T>>;

	fn render(&self, context: &mut Phi);
}

pub trait HitBox {

	// Global CS
	fn frame(&self) -> &Rectangle;

	// Spritesheet-local CS
	fn bounds(&self) -> &Rectangle;


	fn collision_mask(&self) -> &AlphaChannel;

	fn collides_with(&self, another: &HitBox) -> Option<Rectangle> {
		Rectangle::intersection(self.frame(), another.frame())
		.map_or(None, |intersection| {

			if AlphaChannel::intersect(self.collision_mask(), self.frame().x - self.bounds().x, self.frame().y - self.bounds().y, 
				another.collision_mask(), another.frame().x - another.bounds().x, another.frame().y - another.bounds().y, intersection) {

				return Some(intersection);
			}
			None
		})
	}
}


fn load_spritesheet_with_alpha (phi: &Phi, path: &str, alpha_threshold: f64) -> Result<(AlphaChannel, Sprite), ::std::io::Error> {
	let alpha_path = ::std::path::Path::new(path).with_extension("acl0");

	match AlphaChannel::from_file(&alpha_path) {
		Ok(alpha) => { 
			Ok((alpha, Sprite::load(&phi.renderer, path).unwrap()))
		},
		_ => {
			let surface_reader = RWops::from_file(path, "rb").unwrap();
			let surface = surface_reader.load().unwrap();

			let alpha = unsafe { AlphaChannel::from_surface(&surface, Some(alpha_threshold)).unwrap() };

			try!(alpha.save_to(&alpha_path));

			Ok((alpha, Sprite::from_surface(&phi.renderer, &surface).unwrap()))
		}
	}
}


#[cfg(test)]
mod tests {
	extern crate time;

	use super::*;
	use ::phi::{ Phi, View, ViewAction };
	use self::time::{ PreciseTime, Duration };

	struct EmptyView {

	}

	impl View for EmptyView {
		fn update(mut self: Box<Self>, phi: &mut Phi, elapsed: f64) -> ViewAction {
			ViewAction::Quit
		}

		fn render(&self, phi: &mut Phi) {
			/* Nothing to do  */
		}
	}


	fn measure<F>(mut action: F)  -> Duration where F: FnMut() -> () {
		let start = PreciseTime::now();
		{
			action();	
		}
		start.to(PreciseTime::now())
	}


	const GAME_VIEW_LOAD_REPEAT_COUNT: u32 = 1000;

	#[test]
	fn bench_game_view_load() {
		::phi::spawn("testr", (1, 1), |phi| {
			let mut duration_total = 0i64;

			for i in 0..GAME_VIEW_LOAD_REPEAT_COUNT {
				duration_total += measure(|| {
					GameView::new(phi);
				}).num_milliseconds();

				::std::fs::remove_file("assets/sprites/asteroid.acl0");	
				::std::fs::remove_file("assets/sprites/spaceship.acl0");
			}
			println!("GameView initialization takes {}ms in average.", duration_total / GAME_VIEW_LOAD_REPEAT_COUNT as i64);

			Box::new(EmptyView {})
		});
	}
}