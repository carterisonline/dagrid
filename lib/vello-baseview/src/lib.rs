use std::num::{NonZeroIsize, NonZeroU32, NonZeroUsize};
use std::ptr::NonNull;

pub use baseview::{Size, Window, WindowHandle, WindowInfo, WindowOpenOptions, WindowScalePolicy};
use baseview::{WindowEvent, WindowHandler};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use raw_window_handle_06::{
    AppKitDisplayHandle, AppKitWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
    XcbDisplayHandle, XcbWindowHandle, XlibDisplayHandle, XlibWindowHandle,
};
pub use vello;
use vello::peniko::Color;
use vello::util::RenderContext;
use vello::wgpu::{
    Device, Limits, Queue, Surface, SurfaceConfiguration, SurfaceTargetUnsafe, TextureFormat,
};
use vello::{wgpu, AaSupport, RenderParams, Renderer, RendererOptions, Scene};

mod simple_text;
pub use simple_text::SimpleText;

pub struct WindowContext {
    renderer: Renderer,
    scene: Scene,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
}

impl WindowContext {
    pub fn new(window: &mut baseview::Window, window_info: WindowInfo) -> Self {
        let raw_display_handle = window.raw_display_handle();
        let raw_window_handle = window.raw_window_handle();

        let target = SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: match raw_display_handle {
                raw_window_handle::RawDisplayHandle::AppKit(_) => {
                    raw_window_handle_06::RawDisplayHandle::AppKit(AppKitDisplayHandle::new())
                }
                raw_window_handle::RawDisplayHandle::Xlib(handle) => {
                    raw_window_handle_06::RawDisplayHandle::Xlib(XlibDisplayHandle::new(
                        NonNull::new(handle.display),
                        handle.screen,
                    ))
                }
                raw_window_handle::RawDisplayHandle::Xcb(handle) => {
                    raw_window_handle_06::RawDisplayHandle::Xcb(XcbDisplayHandle::new(
                        NonNull::new(handle.connection),
                        handle.screen,
                    ))
                }
                raw_window_handle::RawDisplayHandle::Windows(_) => {
                    raw_window_handle_06::RawDisplayHandle::Windows(WindowsDisplayHandle::new())
                }
                _ => todo!(),
            },
            raw_window_handle: match raw_window_handle {
                raw_window_handle::RawWindowHandle::AppKit(handle) => {
                    raw_window_handle_06::RawWindowHandle::AppKit(AppKitWindowHandle::new(
                        NonNull::new(handle.ns_view).unwrap(),
                    ))
                }
                raw_window_handle::RawWindowHandle::Xlib(handle) => {
                    raw_window_handle_06::RawWindowHandle::Xlib(XlibWindowHandle::new(
                        handle.window,
                    ))
                }
                raw_window_handle::RawWindowHandle::Xcb(handle) => {
                    raw_window_handle_06::RawWindowHandle::Xcb(XcbWindowHandle::new(
                        NonZeroU32::new(handle.window).unwrap(),
                    ))
                }
                raw_window_handle::RawWindowHandle::Win32(handle) => {
                    // will this work? i have no idea!
                    //      CR: windows schmindows who cares
                    let mut raw_handle =
                        Win32WindowHandle::new(NonZeroIsize::new(handle.hwnd as isize).unwrap());

                    raw_handle.hinstance = handle
                        .hinstance
                        .is_null()
                        .then(|| NonZeroIsize::new(handle.hinstance as isize).unwrap());

                    raw_window_handle_06::RawWindowHandle::Win32(raw_handle)
                }
                _ => todo!(),
            },
        };

        let ctx = RenderContext::new();

        let surface = unsafe { ctx.instance.create_surface_unsafe(target) }.unwrap();

        let adapter = pollster::block_on(wgpu::util::initialize_adapter_from_env_or_default(
            &ctx.instance,
            Some(&surface),
        ))
        .unwrap();
        let features = adapter.features();
        let limits = Limits::default();
        let maybe_features = wgpu::Features::CLEAR_TEXTURE;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: features & maybe_features,
                required_limits: limits,
            },
            None,
        ))
        .unwrap();

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .into_iter()
            .find(|it| matches!(it, TextureFormat::Rgba8Unorm | TextureFormat::Bgra8Unorm))
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: window_info.physical_size().width,
            height: window_info.physical_size().height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let renderer = Renderer::new(
            &device,
            RendererOptions {
                surface_format: Some(format),
                use_cpu: false,
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: NonZeroUsize::new(1),
            },
        )
        .unwrap();

        let scene = Scene::new();

        Self {
            renderer,
            scene,
            device,
            queue,
            surface,
            config,
        }
    }
}

pub struct Application<T, U> {
    window_context: WindowContext,
    context: T,
    f: U,
}

impl<T, U> Application<T, U> {
    pub fn new(window_context: WindowContext, context: T, f: U) -> Self {
        Self {
            window_context,
            context,
            f,
        }
    }
}

impl<T, U: FnMut(&mut T, &mut Scene)> WindowHandler for Application<T, U> {
    fn on_frame(&mut self, _window: &mut baseview::Window) {
        self.window_context.scene.reset();

        (self.f)(&mut self.context, &mut self.window_context.scene);

        let surface_texture = self.window_context.surface.get_current_texture().unwrap();

        self.window_context
            .renderer
            .render_to_surface(
                &self.window_context.device,
                &self.window_context.queue,
                &self.window_context.scene,
                &surface_texture,
                &RenderParams {
                    base_color: Color::GREEN,
                    width: self.window_context.config.width,
                    height: self.window_context.config.height,
                    antialiasing_method: vello::AaConfig::Area,
                },
            )
            .unwrap();

        surface_texture.present();
        self.window_context.device.poll(wgpu::MaintainBase::Poll);
    }

    fn on_event(
        &mut self,
        _window: &mut baseview::Window,
        event: baseview::Event,
    ) -> baseview::EventStatus {
        match event {
            baseview::Event::Window(WindowEvent::Resized(new_info)) => {
                self.window_context.config.width = new_info.physical_size().width;
                self.window_context.config.height = new_info.physical_size().height;
                self.window_context
                    .surface
                    .configure(&self.window_context.device, &self.window_context.config);

                baseview::EventStatus::Captured
            }
            _ => baseview::EventStatus::Ignored,
        }
    }
}
