#[macro_use]
mod events;
pub mod data;

use sdl2::render::Renderer;
use sdl2::pixels::Color;

struct_events! {
    keyboard: {
        key_escape: Escape,
        key_up: Up,
        key_down: Down,
        key_left: Left,
        key_right: Right,
        key_space: Space
    },

    else: {
        quit: Quit { .. }
    }
}

pub struct Phi<'window> {
    pub events: Events,
    pub renderer: Renderer<'window>,
}

impl<'window> Phi<'window> {
    pub fn output_size(&self) -> (f64, f64) {
        let (w, h): (u32, u32) = self.renderer.output_size().unwrap();
        (w as f64, h as f64)
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

    // Create the window
    let window = video.window("ArcadeRS Shooter", 800, 600)
        .position_centered()
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let mut context = Phi {
        events: Events::new(sdl_context.event_pump().unwrap()),
        renderer: window.renderer()
            .accelerated()
            .build().unwrap(),
    };

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
