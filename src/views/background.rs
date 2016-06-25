use phi::data::Rectangle;
use phi::gfx::{Renderable, Sprite};

use sdl2::render::Renderer;


#[derive(Clone)]
pub struct Background {
    pos: f64,
    // The amount of pixels moved to the left every second
    vel: f64,
    sprite: Sprite,
}

impl Background {
    pub fn load(renderer: &Renderer, path: &str, velocity: f64) -> Option<Background> {
        Sprite::load(&renderer, path)
        .map(|sprite| {
            Background {
                pos: 0.0,

                vel: velocity,
                sprite: sprite,
            }
        })
    }

    pub fn render(&mut self, renderer: &mut Renderer, elapsed: f64) {
        // We define a logical position as depending solely on the time and the
        // dimensions of the image, not on the screen's size.
        let size = self.sprite.size();
        self.pos += self.vel * elapsed;
        if self.pos > size.0 {
            self.pos -= size.0;
        }

        // We determine the scale ratio of the window to the sprite.
        let (win_w, win_h) = renderer.output_size().unwrap();
        let scale = win_h as f64 / size.1;

        // We render as many copies of the background as necessary to fill
        // the screen.
        let mut physical_left = -self.pos * scale;

        while physical_left < win_w as f64 {
            //? While the left of the image is still inside of the window...
            self.sprite.render(renderer, Rectangle {
                x: physical_left,
                y: 0.0,
                w: size.0 * scale,
                h: win_h as f64,
            });

            physical_left += size.0 * scale;
        }
    }
}
