use phi::{Phi, View, ViewAction};
use phi::data::Rectangle;
use phi::gfx::{Sprite, AnimatedSprite, CopySprite};
use sdl2::pixels::Color;
use sdl2::render::Renderer;
use views::shared::Background;

/// Pixels traveled by the player's ship every second, when it's moving
const PLAYER_SPEED: f64 = 180.0;

const SHIP_W: f64 = 43.0;
const SHIP_H: f64 = 39.0;

const DEBUG: bool = false;

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

    asteroid: Asteroid,

    bg_back: Background,
    bg_middle: Background,
    bg_front: Background,
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
            },

            asteroid: Asteroid::new(phi),

            bg_back: Background {
                pos: 0.0,
                vel: 20.0,
                sprite: Sprite::load(&phi.renderer, "assets/starBG.png").unwrap(),
            },

            bg_middle: Background {
                pos: 0.0,
                vel: 40.0,
                sprite: Sprite::load(&phi.renderer, "assets/starMG.png").unwrap(),
            },

            bg_front: Background {
                pos: 0.0,
                vel: 80.0,
                sprite: Sprite::load(&phi.renderer, "assets/starFG.png").unwrap(),
            },
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


        // Update the asteroid
        self.asteroid.update(phi, elapsed);

        // Clear the screen
        phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
        phi.renderer.clear();

        // Render the Background
        self.bg_back.render(&mut phi.renderer, elapsed);
        self.bg_middle.render(&mut phi.renderer, elapsed);

        // Render the bounding box (for debugging)
        if DEBUG {
            phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
            phi.renderer.fill_rect(self.player.rect.to_sdl().unwrap());
        }

        // Render the ship texture
        phi.renderer.copy_sprite(
            &self.player.sprites[self.player.current as usize],
            self.player.rect,
        );

        // Render the asteroid
        self.asteroid.render(phi);

        // Render the front Background
        self.bg_middle.render(&mut phi.renderer, elapsed);

        ViewAction::None
    }
}


const ASTEROID_PATH: &'static str = "assets/asteroid.png";
const ASTEROIDS_WIDE: usize = 21;
const ASTEROIDS_HIGH: usize = 7;
const ASTEROIDS_TOTAL: usize = ASTEROIDS_WIDE * ASTEROIDS_HIGH - 4;
const ASTEROIDS_SIDE: f64 = 96.0;

struct Asteroid {
    sprite: AnimatedSprite,
    rect: Rectangle,
    vel: f64,
}

impl Asteroid {
    fn new(phi: &mut Phi) -> Asteroid {
        let mut asteroid = Asteroid {
            sprite: Asteroid::get_sprite(phi, 15.0),
            rect: Rectangle {
                x: 128.0,
                y: 128.0,
                w: ASTEROIDS_SIDE,
                h: ASTEROIDS_SIDE,
            },
            vel: 0.0,
        };

        asteroid.reset(phi);
        asteroid
    }

    fn update(&mut self, phi: &mut Phi, dt: f64) {
        self.rect.x -= dt * self.vel;
        self.sprite.add_time(dt);

        if self.rect.x <= -ASTEROIDS_SIDE {
            self.reset(phi);
        }
    }

    fn render(&self, phi: &mut Phi) {
        phi.renderer.copy_sprite(&self.sprite, self.rect);
    }

    fn reset(&mut self, phi: &mut Phi) {
        let (w, h) = phi.output_size();

        // FPS between 10.0 and 30.0
        self.sprite.set_fps(::rand::random::<f64>().abs() * 20.0 + 10.0);

        self.rect = Rectangle {
            x: w,
            y: ::rand::random::<f64>().abs() * (h - ASTEROIDS_SIDE),
            w: ASTEROIDS_SIDE,
            h: ASTEROIDS_SIDE,
        };

        // vel between 50.0 and 150.0
        self.vel = ::rand::random::<f64>().abs() * 100.0 + 50.0;
    }

    fn get_sprite(phi: &mut Phi, fps: f64) -> AnimatedSprite {
        let asteroid_spritesheet = Sprite::load(&mut phi.renderer, ASTEROID_PATH).unwrap();
        let mut asteroid_sprites = Vec::with_capacity(ASTEROIDS_TOTAL);

        for yth in 0..ASTEROIDS_HIGH {
            for xth in 0..ASTEROIDS_WIDE {
                // There are four asteroids missing at the end of the sprite. We don't want those.
                if ASTEROIDS_WIDE * yth + xth >= ASTEROIDS_TOTAL {
                    break;
                }

                asteroid_sprites.push(
                    asteroid_spritesheet.region(Rectangle {
                        x: xth as f64 * ASTEROIDS_SIDE,
                        y: yth as f64 * ASTEROIDS_SIDE,
                        w: ASTEROIDS_SIDE,
                        h: ASTEROIDS_SIDE,
                    }).unwrap()
                );
            }
        }

        AnimatedSprite::with_fps(asteroid_sprites, fps)
    }
}