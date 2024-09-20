use std::sync::{Arc, RwLock};

use nih_plug::editor::Editor;
use nih_plug::prelude::GuiContext;
use vello_baseview::vello::Scene;
use vello_baseview::{
    Application, Window, WindowContext, WindowHandle, WindowInfo, WindowOpenOptions,
    WindowScalePolicy,
};

pub struct VelloContext<T, U> {
    pub user: T,
    pub editor: U,
}

pub(crate) struct VelloEditor<T, U> {
    pub(crate) context: Arc<RwLock<VelloContext<T, Option<Arc<dyn GuiContext>>>>>,
    pub(crate) update: U,
    pub(crate) window_info: WindowInfo,
}

struct VelloEditorHandle(WindowHandle);

unsafe impl Send for VelloEditorHandle {}
impl Drop for VelloEditorHandle {
    fn drop(&mut self) {
        self.0.close();
    }
}

impl<T, U> Editor for VelloEditor<T, &'static U>
where
    T: 'static + Send + Sync,
    U: Fn(&mut Arc<RwLock<VelloContext<T, Option<Arc<dyn GuiContext>>>>>, &mut Scene)
        + 'static
        + Send
        + Sync,
{
    fn spawn(
        &self,
        parent: nih_plug::editor::ParentWindowHandle,
        context: Arc<dyn GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        let window_open_options = WindowOpenOptions {
            title: "vello window".into(),
            size: self.window_info.logical_size(),
            #[cfg(target_os = "macos")]
            scale: WindowScalePolicy::SystemScaleFactor,
            #[cfg(not(target_os = "macos"))]
            scale: WindowScalePolicy::ScaleFactor(self.window_info.scale()),
            gl_config: None,
        };

        self.context.write().unwrap().editor = Some(context.clone());
        let full_context = self.context.clone();
        let update = self.update;
        let window_info = self.window_info;

        let window = Window::open_parented(&parent, window_open_options, move |window| {
            Application::new(
                WindowContext::new(window, window_info),
                full_context,
                update,
            )
        });

        Box::new(VelloEditorHandle(window))
    }

    fn size(&self) -> (u32, u32) {
        let size = self.window_info.physical_size();
        (size.width, size.height)
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        false
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {
        ()
    }

    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {
        ()
    }

    fn param_values_changed(&self) {
        ()
    }
}
