#[macro_use]
mod events;

use sdl2::render::Renderer;

struct_events! {
    keyboard: {
        key_escape: Escape,
        key_up: Up,
        key_down: Down
    },

    else: {
        quit: Quit { .. }
    }
}

pub struct Phi<'window> {
    pub events: Events,
    pub renderer: Renderer<'window>,
}

/// `ViewAction` allows the current view to tell the render loop what shpuld
/// happen next.
pub enum ViewAction {
    None,
    Quit,
}

pub trait View {
    /// Renders the current view and returns a `ViewAction` so it can communicate
    /// with the render loop. Called on every frame.
    // TODO: separate update and render logic?
    fn render(&mut self, context: &mut Phi, elapsed: f64) -> ViewAction;
}
