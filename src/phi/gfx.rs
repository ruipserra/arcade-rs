use phi::data::Rectangle;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use sdl2::render::{Renderer, Texture};
use sdl2_image::LoadTexture;

pub trait Renderable {
    fn render(&self, renderer: &mut Renderer, dest: Rectangle);
}

#[derive(Clone)]
pub struct Sprite {
    tex: Rc<RefCell<Texture>>,
    src: Rectangle,
}

impl Sprite {
    /// Creates a new sprite by wrapping `Texture`
    pub fn new(texture: Texture) -> Sprite {
        let tex_query = texture.query();

        Sprite {
            tex: Rc::new(RefCell::new(texture)),
            src: Rectangle {
                x: 0.0,
                y: 0.0,
                w: tex_query.width as f64,
                h: tex_query.height as f64,
            }
        }
    }

    /// Creates a new sprite from an image file located at the given path.
    /// Returns `Some(Sprite)` if the file could be read, `None` otherwise.
    pub fn load(renderer: &Renderer, path: &str) -> Option<Sprite> {
        renderer.load_texture(Path::new(path)).ok().map(Sprite::new)
    }

    /// Returns a new `Sprite` representing a sub-region of the current one.
    /// The provided `rect` is relative to the currently held region.
    /// Returns `Some(Sprite)` if `rect` is valid, i.e. included in the current
    /// region; returns `None` otherwise.
    pub fn region(&self, rect: Rectangle) -> Option<Sprite> {
        let new_src = Rectangle {
            x: self.src.x + rect.x,
            y: self.src.y + rect.y,
            ..rect
        };

        if self.src.contains(new_src) {
            Some(Sprite {
                tex: self.tex.clone(),
                src: new_src,
            })
        } else {
            None
        }
    }

    /// Returns the dimensions of the region.
    pub fn size(&self) -> (f64, f64) {
        (self.src.w, self.src.h)
    }
}

impl Renderable for Sprite {
    fn render(&self, renderer: &mut Renderer, dest: Rectangle) {
        renderer.copy(&mut self.tex.borrow_mut(), self.src.to_sdl(), dest.to_sdl());
    }
}

pub struct AnimatedSprite {
    /// The frames that wil be rendered, in order
    sprites: Rc<Vec<Sprite>>,

    /// The time it takes to get from one frame to the other, in seconds
    frame_delay: f64,

    /// The total time the frame has been alive, from which the current frame is derived
    current_time: f64,
}

impl AnimatedSprite {
    /// Creates a new animated sprite initialized at time 0
    pub fn new(sprites: Vec<Sprite>, frame_delay: f64) -> AnimatedSprite {
        AnimatedSprite {
            sprites: Rc::new(sprites),
            frame_delay: frame_delay,
            current_time: 0.0,
        }
    }

    /// Creates a new animated sprite which goes to the next frame `fps` times every second.
    pub fn with_fps(sprites: Vec<Sprite>, fps: f64) -> AnimatedSprite {
        if fps == 0.0 {
            panic!("Passed 0 to AnimatedSprite::with_fps");
        }

        AnimatedSprite::new(sprites, 1.0 / fps)
    }

    /// The number of frames composing the animation
    pub fn frames(&self) -> usize {
        self.sprites.len()
    }

    /// Set the time it takes to get from one frame to the next one, in second.
    /// If the time is negative, then we rewind the animation.
    pub fn set_frame_delay(&mut self, frame_delay: f64) {
        self.frame_delay = frame_delay;
    }

    /// Set the number of frames the animation goes through every second.
    /// If the value is negative, then we rewind the animation.
    pub fn set_fps(&mut self, fps: f64) {
        if fps == 0.0 {
            panic!("Passed 0 to AnimatedSprite::set_fps");
        }

        self.set_frame_delay(1.0 / fps);
    }

    /// Adds a certain amount of time, in seconds, to the `current_time` of the the
    /// animated sprite, so that it knows when to go to the next frame.
    pub fn add_time(&mut self, dt: f64) {
        self.current_time += dt;

        // If we decide to "go back in time", this allows us to select the last
        // frame whenever we reach a negative one.
        if self.current_time < 0.0 {
            self.current_time = (self.frames() - 1) as f64 * self.frame_delay;
        }
    }
}

impl Renderable for AnimatedSprite {
    /// Renders the current frame of the sprite
    fn render(&self, renderer: &mut Renderer, dest: Rectangle) {
        let current_frame = (self.current_time / self.frame_delay) as usize % self.frames();
        let sprite = &self.sprites[current_frame];
        sprite.render(renderer, dest);
    }
}

pub trait CopySprite<T> {
    fn copy_sprite(&mut self, sprite: &T, dest: Rectangle);
}

impl<'window, T: Renderable> CopySprite<T> for Renderer<'window> {
    fn copy_sprite(&mut self, renderable: &T, dest: Rectangle) {
        renderable.render(self, dest);
    }
}