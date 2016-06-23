use phi::{Phi, View, ViewAction};
use phi::data::Rectangle;
use phi::gfx::Sprite;

use sdl2::pixels::Color;
use sdl2::render::Renderer;


// Constants
const PLAYER_SPEED: f64 = 180.0;

const SHIP_W: f64 = 64.0;
const SHIP_H: f64 = 64.0;


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
struct Ship {
    rect: Rectangle,
    sprites: Vec<Sprite>,
    current: ShipFrame,
}

#[derive(Clone)]
struct Background {
    pos: f64,
    // The amount of pixels moved to the left every second
    vel: f64,
    sprite: Sprite,
}

impl Background {
    fn render(&mut self, renderer: &mut Renderer, elapsed: f64) {
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


pub struct ShipView {
    player: Ship,

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
            },

            bg_back: Background {
                pos: 0.0,
                vel: 20.0,
                sprite: Sprite::load(&mut phi.renderer, "assets/starBG.png").unwrap(),
            },
            bg_middle: Background {
                pos: 0.0,
                vel: 40.0,
                sprite: Sprite::load(&mut phi.renderer, "assets/starMG.png").unwrap(),
            },
            bg_front: Background {
                pos: 0.0,
                vel: 80.0,
                sprite: Sprite::load(&mut phi.renderer, "assets/starFG.png").unwrap(),
            },
        }
    }
}

impl View for ShipView {
    fn render(&mut self, phi: &mut Phi, elapsed: f64) -> ViewAction {
        if phi.events.now.quit || phi.events.now.key_escape == Some(true) {
            return ViewAction::Quit;
        }

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

        // Clear the screen
        phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
        phi.renderer.clear();

        // Render Backgrounds
        self.bg_back.render(&mut phi.renderer, elapsed);
        self.bg_middle.render(&mut phi.renderer, elapsed);

        // Render the bounding box (for debugging purposes)
        if DEBUG {
            phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
            phi.renderer.fill_rect(self.player.rect.to_sdl().unwrap());
        }
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

        self.player.sprites[self.player.current as usize]
        .render(&mut phi.renderer, self.player.rect);

        // Render the foreground
        self.bg_front.render(&mut phi.renderer, elapsed);

        ViewAction::None
    }
}