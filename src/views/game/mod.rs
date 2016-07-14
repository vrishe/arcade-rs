extern crate rand;


mod asteroid;
mod blast;
mod bullet;
mod explosion;
mod player;


use phi::{Phi, View, ViewAction};
use phi::data::{MaybeAlive, Rectangle};
use phi::gfx::{AlphaChannel, Renderable, Sprite};

use sdl2::pixels::Color;
use sdl2::rwops::RWops;
use sdl2_image::ImageRWops;
use sdl2_mixer::Chunk;

use std::path::Path;

use views::background::Background;


use self::asteroid::{Asteroid, AsteroidFactory};
use self::blast::Blast;
use self::bullet::Bullet;
use self::explosion::{Explosion, ExplosionFactory};
use self::player::Player;


const SHOT_DELAY: f64 = 1.0 / 7.62;


pub struct GameView {
	player: Player,
	shot_time: f64,

	asteroid_factory: AsteroidFactory,
	explosion_factory: ExplosionFactory,

	asteroids: Vec<Asteroid>,
	blasts: Vec<Box<Blast>>,
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
			blasts: vec![],
			bullets: vec![],
			explosions: vec![],

			bg_ambient: Background::load(&phi.renderer, "assets/backgrounds/starAMB.png", 0.0).unwrap(),
			bg_back: Background::load(&phi.renderer, "assets/backgrounds/starBG.png", 20.0).unwrap(),
			bg_middle: Background::load(&phi.renderer, "assets/backgrounds/starMG.png", 40.0).unwrap(),
			bg_front: Background::load(&phi.renderer, "assets/backgrounds/starFG.png", 80.0).unwrap(),

			bullet_sound: Chunk::from_file(Path::new("assets/sounds/bullet.ogg")).unwrap(),
			explosion_sound: Chunk::from_file(Path::new("assets/sounds/explosion.ogg")).unwrap()
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
					return asteroid.update(elapsed);
				}
				game.explosions.push(
					game.explosion_factory.at_center(
						asteroid.rect().center()));

				phi.play_sound(&game.explosion_sound);

				None
			})
			.collect();

			// TODO
			// For the moment, we won't do anything about the player dying. This will be
			// the subject of a future episode.
			if !player_alive {
				println!("The player's Ship has been destroyed.");
			}
			game.player.update(phi, elapsed);

			// Update the explosions
			game.explosions = ::std::mem::replace(&mut game.explosions, vec![]).into_iter()
			.filter_map(|explosion| explosion.update(elapsed))
			.collect();

			//? Upon assignment, the old value of `self.bullets`, namely the empty vector,
			//? will be freed automatically, because its owner no longer refers to it.
			//? We can then update the bullet quite simply.
			game.bullets = transition_bullets.into_iter()
			.filter_map(|transition_bullet| {
				match transition_bullet.as_option() {
					Some(bullet) => {
						let center = bullet.rect().center();

						for explosion in &game.explosions {
							if explosion.blast(center) {
								return None;
							}
						}
						bullet.update(phi, elapsed)
					}
					_ => None
				}
			})
			.collect();

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


trait GameObject<T> {

	fn location(&self) -> (f64, f64);


	fn update(mut self: Box<Self>, context: &mut Phi, dt: f64) -> Option<Box<T>>;

	fn render(&self, context: &mut Phi);
}

trait HitBox {

	// Global CS
	fn frame(&self) -> &Rectangle;

	// Spritesheet-local CS
	fn bounds(&self) -> &Rectangle;


	fn collision_mask(&self) -> &AlphaChannel;

	fn collides_with(&self, another: &HitBox) -> bool {
		Rectangle::intersection(self.frame(), another.frame())
		.map_or(false, |intersection| {

			AlphaChannel::intersect(self.collision_mask(), self.frame().x - self.bounds().x, self.frame().y - self.bounds().y, 
				another.collision_mask(), another.frame().x - another.bounds().x, another.frame().y - another.bounds().y, intersection)

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