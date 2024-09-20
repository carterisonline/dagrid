use std::sync::{Arc, RwLock};

use nih_plug::context::gui::GuiContext;
use nih_plug::editor::Editor;
use vello_baseview::vello::Scene;
use vello_baseview::WindowInfo;
pub use vello_baseview::{vello, SimpleText, Size};

pub use crate::editor::VelloContext;

mod editor;

pub fn create_vello_editor<
    T: 'static + Send + Sync,
    U: Fn(&mut Arc<RwLock<VelloContext<T, Option<Arc<dyn GuiContext>>>>>, &mut Scene)
        + 'static
        + Send
        + Sync,
>(
    size: Size,
    context: T,
    f: &'static U,
) -> Option<Box<dyn Editor>> {
    Some(Box::new(editor::VelloEditor {
        context: Arc::new(RwLock::new(editor::VelloContext {
            user: context,
            editor: None,
        })),
        update: f,
        window_info: WindowInfo::from_logical_size(size, 1.0),
    }))
}
