use phi::{Phi, View, ViewAction};
use phi::data::Rectangle;
use phi::gfx::{Sprite, CopySprite};
use sdl2::pixels::Color;

/// Pixels traveled by the player's ship every second, when it's moving
const PLAYER_SPEED: f64 = 180.0;

const SHIP_W: f64 = 43.0;
const SHIP_H: f64 = 39.0;

/// The different states our ship can be in. In the image, they're ordered
/// from left to right, top to bottom.
#[derive(Copy, Clone)]
enum ShipFrame {
    UpNorm   = 0,
    UpFast   = 1,
    UpSlow   = 2,
    MidNorm  = 3,
    MidFast  = 4,
    MidSlow  = 5,
    DownNorm = 6,
    DownFast = 7,
    DownSlow = 8,
}

struct Ship {
    rect: Rectangle,
    sprites: Vec<Sprite>,
    current: ShipFrame,
}


pub struct ShipView {
    player: Ship,
}

impl ShipView {
    pub fn new(phi: &mut Phi) -> ShipView {
        let spritesheet = Sprite::load(&phi.renderer, "assets/spaceship.png").unwrap();

        let mut sprites = Vec::with_capacity(9);

        for y in 0..3 {
            for x in 0..3 {
                sprites.push(spritesheet.region(Rectangle {
                    x: SHIP_W * x as f64,
                    y: SHIP_H * y as f64,
                    w: SHIP_W,
                    h: SHIP_H,
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
            }
        }
    }
}

impl View for ShipView {
    fn render(&mut self, phi: &mut Phi, elapsed: f64) -> ViewAction {
        if phi.events.now.quit || phi.events.now.key_escape == Some(true) {
            return ViewAction::Quit;
        }

        // let traveled = PLAYER_SPEED * elapsed;
        let diagonal =
            (phi.events.key_up ^ phi.events.key_down) &&
            (phi.events.key_left ^ phi.events.key_right);

        let moved =
            if diagonal { 1.0/2.0f64.sqrt() }
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

        self.player.current =
            if dy < 0.0 {
                if dx < 0.0 { ShipFrame::UpSlow }
                else if dx == 0.0 { ShipFrame::UpNorm }
                else { ShipFrame::UpFast }
            } else if dy == 0.0 {
                if dx < 0.0 { ShipFrame::MidSlow }
                else if dx == 0.0 { ShipFrame::MidNorm }
                else { ShipFrame::MidFast }
            } else {
                if dx < 0.0 { ShipFrame::DownSlow }
                else if dx == 0.0 { ShipFrame::DownNorm }
                else { ShipFrame::DownFast }
            };

        // The movable region spans the entire window height, and 70%
        // of the window's width.
        let (screen_w, screen_h) = phi.output_size();
        let movable_region = Rectangle {
            x: 0.0,
            y: 0.0,
            w: screen_w * 0.7,
            h: screen_h,
        };

        self.player.rect = self.player.rect.move_inside(movable_region).unwrap();

        // Clear the screen
        phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
        phi.renderer.clear();

        // Render the bounding box (for debugging)
        // phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
        // phi.renderer.fill_rect(self.player.rect.to_sdl().unwrap());

        // Render the ship texture
        phi.renderer.copy_sprite(
            &self.player.sprites[self.player.current as usize],
            self.player.rect,
        );

        ViewAction::None
    }
}