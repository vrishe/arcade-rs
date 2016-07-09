use phi::{Phi, View, ViewAction};
use phi::data::Rectangle;
use phi::gfx::{Renderable, Sprite};

use sdl2::pixels::Color;
use sdl2_mixer::Music;

use views::background::Background;


pub struct MainMenuView {
    actions: Vec<Action>,
    //? We're using i8 instead of usize (0..) so that we do not have underflow
    //? errors when decrementing it on key_up.
    selected: i8,

    time: f64,
    logo: Sprite,
    sprite: Sprite,

    bg_back: Background,

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
            bg_back: Background::load(&phi.renderer, "assets/backgrounds/starBG.png", 32.0).unwrap(),

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
        self.bg_back.update(elapsed);
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
        let scale = (win_h / size.1) * ((self.time * 45e-4).sin() + 1.0);

        // We define a logical position as depending solely on the time and the
        // dimensions of the image, not on the screen's size.
        let sprite_w = size.0 * scale;
        let sprite_h = size.1 * scale;

        self.sprite.render(&mut phi.renderer, Rectangle {
            x: (win_w - sprite_w) * 0.5,
            y: (win_h - sprite_h) * 0.5,
            w: sprite_w,
            h: sprite_h,
        });
        self.bg_back.render(&mut phi.renderer);

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