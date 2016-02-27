#[macro_use]
mod events;
pub mod data;
pub mod gfx;

use sdl2::render::Renderer;
use sdl2::pixels::Color;
use std::path::Path;
use std::collections::HashMap;
use self::gfx::Sprite;

struct_events! {
    keyboard: {
        key_escape: Escape,
        key_up: Up,
        key_down: Down,
        key_left: Left,
        key_right: Right,
        key_space: Space,
        key_enter: Enter
    },

    else: {
        quit: Quit { .. }
    }
}

pub struct Phi<'window> {
    pub events: Events,
    pub renderer: Renderer<'window>,

    cached_fonts: HashMap<(&'static str, i32), ::sdl2_ttf::Font>,
}

impl<'window> Phi<'window> {
    fn new(events: Events, renderer: Renderer<'window>) -> Phi<'window> {
        Phi {
            events: events,
            renderer: renderer,
            cached_fonts: HashMap::new(),
        }
    }

    pub fn output_size(&self) -> (f64, f64) {
        let (w, h): (u32, u32) = self.renderer.output_size().unwrap();
        (w as f64, h as f64)
    }

    /// Renders a string of text as a sprite using the provided parameters.
    pub fn ttf_str_sprite(&mut self, text: &str, font_path: &'static str, size: i32, color: Color) -> Option<Sprite> {
        // First, we check if the font is already cached. If this is the case,
        // we use it to render the text.
        if let Some(font) = self.cached_fonts.get(&(font_path, size)) {
            return font.render(text, ::sdl2_ttf::blended(color)).ok()
                .and_then(|surface| self.renderer.create_texture_from_surface(&surface).ok())
                .map(Sprite::new);
        }

        // Otherwise, we start by trying to load the requested font.
        ::sdl2_ttf::Font::from_file(Path::new(font_path), size).ok()
            .and_then(|font| {
                // If this worked, we cache the font we acquired.
                self.cached_fonts.insert((font_path, size), font);

                // Then, we call this method recursively. This avoids repeating
                // the rendering code.
                self.ttf_str_sprite(text, font_path, size, color)
            })
    }
}

/// `ViewAction` allows the current view to tell the render loop what shpuld
/// happen next.
pub enum ViewAction {
    None,
    Quit,
    ChangeView(Box<View>),
}

pub trait View {
    /// Renders the current view and returns a `ViewAction` so it can communicate
    /// with the render loop. Called on every frame.
    // TODO: separate update and render logic?
    fn render(&mut self, context: &mut Phi, elapsed: f64) -> ViewAction;
}


/// Create a window with name `title`, initialize the underlying libraries and
/// start the game with the view returned by `init()`.
///
/// # Examples
///
/// Here, we simply show a window with color #ffff00 and exit when escape is
/// pressed or when the window is closed.
///
/// ```
/// struct MyView;
/// impl View for MyView {
///     fn render(&mut self, context: &mut Phi, _: f64) -> ViewAction {
///         if context.events.now.quit {
///             return ViewAction::Quit;
///         }
///
///         context.renderer.set_draw_color(Color::RGB(255, 255, 0));
///         context.renderer.clear();
///         ViewAction::None
///     }
/// }
///
/// spawn("Example", |_| Box::new(MyView));
/// ```
pub fn spawn<F>(title: &str, init: F)
    where F: Fn(&mut Phi) -> Box<View> {

    // Initizalize SDL2
    let sdl_context = ::sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let mut timer = sdl_context.timer().unwrap();
    let _image_context = ::sdl2_image::init(::sdl2_image::INIT_PNG).unwrap();
    let _ttf_context = ::sdl2_ttf::init().unwrap();

    // Create the window
    let window = video.window("ArcadeRS Shooter", 800, 600)
        .position_centered()
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let mut context = Phi::new(
        Events::new(sdl_context.event_pump().unwrap()),
        window.renderer()
            .accelerated()
            .build().unwrap()
    );

    let mut current_view = init(&mut context);

    let interval = 1_000 / 60;
    let mut before = timer.ticks();
    let mut last_second = timer.ticks();
    let mut fps = 0u16;

    loop {
        let now = timer.ticks();
        let dt = now - before;
        let elapsed = dt as f64 / 1_000.0;

        if dt < interval {
            timer.delay(interval - dt);
            continue;
        }

        before = now;
        fps += 1;

        if now - last_second > 1_000 {
            println!("FPS: {}", fps);
            last_second = now;
            fps = 0;
        }

        context.events.pump(&mut context.renderer);

        match current_view.render(&mut context, 0.01) {
            ViewAction::None => context.renderer.present(),
            ViewAction::Quit => break,
            ViewAction::ChangeView(new_view) => current_view = new_view,
        }
    }
}
