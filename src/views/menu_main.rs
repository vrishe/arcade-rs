extern crate rand;


use phi::{Phi, View, ViewAction};
use phi::data::{Rectangle, Point3};
use phi::gfx::{Renderable, Sprite};

use sdl2::pixels::Color;
use sdl2::rect::Point as SdlPoint;
use sdl2_mixer::Music;


const COUNT_STARS_MAX: usize = 2048;
const COUNT_STARS_LIM: usize = (COUNT_STARS_MAX as f64 * 0.095) as usize;
const VELOCITY_STAR: f64 = 48.0;


pub struct MainMenuView {
	actions: Vec<Action>,
	//? We're using i8 instead of usize (0..) so that we do not have underflow
	//? errors when decrementing it on key_up.
	selected: i8,

	time: f64,
	logo: Sprite,
	sprite: Sprite,

	stars: Vec<Point3>,
	stars_frame_buffer: [SdlPoint; COUNT_STARS_MAX],

	music: Music,
}

impl MainMenuView {
	pub fn new(phi: &mut Phi) -> MainMenuView {

		let result = MainMenuView {
			actions: vec![
			Action::new(phi, "New Game", Box::new(|phi| {
				ViewAction::Render(Box::new(::views::game::GameView::new(phi)))
			})),
			Action::new(phi, "Quit", Box::new(|_| {
				ViewAction::Quit
			})),
			],
			//? Start with the option at the top of the screen, with index 0.
			selected: 0,

			time: 0.0,
			logo: Sprite::load(&mut phi.renderer, "assets/logo.png").unwrap(),
			sprite: Sprite::load(&mut phi.renderer, "assets/backgrounds/starAMB.png").unwrap(),

			stars: Vec::with_capacity(COUNT_STARS_MAX),
			stars_frame_buffer: [SdlPoint::new(0, 0); COUNT_STARS_MAX],

			music: Music::from_file(
				::std::path::Path::new("assets/sounds/mdk_phoenix_orchestral.ogg")).unwrap()
		};
		result.music.play(-1).unwrap();
		result
	}
}

impl View for MainMenuView {
	fn update(mut self: Box<Self>,phi: &mut Phi, elapsed: f64) -> ViewAction {
		if phi.events.now.quit || phi.events.now.key_escape == Some(true) {
			return ViewAction::Quit;
		}
		// Execute the currently selected option.
		if phi.events.now.key_return == Some(true) || phi.events.now.key_space == Some(true) {
			//? We must use the (self.attr_which_by_the_way_is_a_closure)(phi)
			//? syntax so that Rust doesn't confuse it with the invocation of
			//? a method called `func`.
			//?
			//? This is necessary because Rust allows a method to share the same
			//? name as an attribute -- a feature which is useful for defining
			//? accessors.
			return (self.actions[self.selected as usize].func)(phi);
		}
		// Change the selected action using the keyboard.
		if phi.events.now.key_up == Some(true) {
			self.selected -= 1;
			//? If we go past the value at the top of the list, we go 'round
			//? to the bottom.
			if self.selected < 0 {
				self.selected = self.actions.len() as i8 - 1;
			}
		}
		if phi.events.now.key_down == Some(true) {
			self.selected += 1;
			//? If we go past the value at the bottom of the list, we go 'round
			//? to the top.
			if self.selected >= self.actions.len() as i8 {
				self.selected = 0;
			}
		}
		self.stars = ::std::mem::replace(&mut self.stars, vec![]).into_iter()
		.filter_map(|mut star| {
			if star.z < -1.0 {
				star.z += elapsed * VELOCITY_STAR;

				return Some(star);
			}
			None
		}).collect();

		let output_size = phi.output_size();
		let center = (output_size.0 * 0.5, output_size.1 * 0.5);
		let mut stars_shortage = COUNT_STARS_MAX - self.stars.len();

		if stars_shortage > COUNT_STARS_LIM {
			let mut rng = self::rand::thread_rng();
			let depth = self::rand::distributions::Range::new(0.0, 1.0);
			let plane = self::rand::distributions::Normal::new(0.0, 1.0);

			while stars_shortage > 0 {
				use self::rand::distributions::IndependentSample;

				self.stars.push(Point3 {
					x: center.0 * plane.ind_sample(&mut rng),
					y: center.1 * plane.ind_sample(&mut rng),
					z: -6.46 * ((output_size.0 * output_size.1) as usize / COUNT_STARS_MAX) as f64 * depth.ind_sample(&mut rng),
				});
				stars_shortage -= 1;
			}
		}
		for i in 0..self.stars.len() {
			let mut star = self.stars[i].projected(1.0);

			star.x = (star.x + 0.5) * output_size.0;
			star.y = (star.y + 0.5) * output_size.1;

			self.stars_frame_buffer[i] = star.to_sdl();
		}
		self.time += elapsed;

		ViewAction::Render(self)
	}

	fn render(&self, phi: &mut Phi) {
		// Clear the screen.
		phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
		phi.renderer.clear();

		let (win_w, win_h) = phi.output_size();
		let size = self.sprite.size();
		// We determine the scale ratio of the window to the sprite.
		let scale = ((win_h / size.1) * ((self.time * 45e-4).sin() + 1.0)) as f32;

		// We define a logical position as depending solely on the time and the
		// dimensions of the image, not on the screen's size.
		let sprite_w = size.0;// * scale;
		let sprite_h = size.1;// * scale;

		phi.renderer.set_scale(scale, scale).unwrap();
		self.sprite.render(&mut phi.renderer, Rectangle {
			x: (win_w - sprite_w) * 0.5,
			y: (win_h - sprite_h) * 0.5,
			w: sprite_w,
			h: sprite_h,
		});
		phi.renderer.set_scale(1f32, 1f32).unwrap();
		phi.renderer.set_draw_color(Color::RGB(170, 172, 181));
		phi.renderer.draw_points(&self.stars_frame_buffer[..self.stars.len()]).unwrap();

		let size = self.logo.size();
		self.logo.render(&mut phi.renderer, Rectangle {
			x: (win_w - size.0) * 0.5,
			y: 40.0,
			w: size.0,
			h: size.1,
		});

		let menu_margin_top = size.1 + 80.0;

		// Render the labels in the menu
		for (i, action) in self.actions.iter().enumerate() {
			if self.selected as usize == i {
				let (w, h) = action.hover_sprite.size();

				let sprite_w = w * ((self.time * 6.0).sin().abs() * 0.16 + 1.0);
				let sprite_x = (win_w - sprite_w) * 0.5;

				action.hover_sprite.render(&mut phi.renderer, Rectangle {
					//? I suggest trying to draw this on a sheet of paper.
					x: sprite_x,
					y: menu_margin_top + 48.0 * i as f64,
					w: sprite_w,
					h: h,
				});
			} else {
				let (w, h) = action.idle_sprite.size();
				action.idle_sprite.render(&mut phi.renderer, Rectangle {
					x: (win_w - w) * 0.5,
					//? We place every element under the previous one.
					y: menu_margin_top + 48.0 * i as f64,
					w: w,
					h: h,
				});
			}
		}
	}
}


struct Action {
	/// The function which should be executed if the action is chosen.
	//? We store it in a Box because, as we saw previously, `Fn` is a trait,
	//? and we may only interact with unsized data through a pointer.
	func: Box<Fn(&mut Phi) -> ViewAction>,

	/// The sprite which is rendered when the player does not focus on this
	/// action's label.
	idle_sprite: Sprite,

	/// The sprite which is rendered when the player "focuses" a label with the
	/// directional keys.
	hover_sprite: Sprite,
}

impl Action {
	fn new(phi: &mut Phi, label: &'static str, func: Box<Fn(&mut Phi) -> ViewAction>) -> Action {
		Action {
			func: func,
			idle_sprite: phi.ttf_str_sprite(label, "assets/fonts/BlackOpsOne-Regular.ttf", 34, Color::RGB(97, 132, 162)).unwrap(),
			hover_sprite: phi.ttf_str_sprite(label, "assets/fonts/BlackOpsOne-Regular.ttf", 38, Color::RGB(255, 255, 0)).unwrap(),
		}
	}
}