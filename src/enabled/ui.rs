use std::borrow::Borrow;

use egui_winit::winit::{
    self,
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};
use parking_lot::Once;

#[macro_export]
macro_rules! init_on {
    ($resources:ident, $loop:expr, $instance:expr, $adapter:expr, $device:expr, $queue:expr) => {
        let mut $resources =
            $crate::enabled::ui::UiResources::new($loop, $instance, $adapter, $device, $queue);
    };
}

#[macro_export]
macro_rules! feed_on {
    ($resources:ident, $event:expr, $control_flow:expr) => {
        $resources.handle_event($event, $control_flow);
    };
}

static UI_LAUNCHED: Once = Once::new();

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn try_launch_own_thread() {
    UI_LAUNCHED.call_once(|| {
        log::info!("This platform does not support creating an event loop on a separate thread")
    })
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
pub fn try_launch_own_thread() {
    UI_LAUNCHED.call_once(|| {
        std::thread::spawn(ui_thread);
    })
}

pub struct UiResources<W, D, Q> {
    window: W,
    device: D,
    queue: Q,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    size: PhysicalSize<u32>,
    ctx: egui::Context,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
}

impl<D, Q> UiResources<Window, D, Q>
where
    D: Borrow<wgpu::Device>,
    Q: Borrow<wgpu::Queue>,
{
    fn new<T>(
        event_loop: &mut winit::event_loop::EventLoop<T>,
        instance: &wgpu::Instance,
        adapter: &wgpu::Adapter,
        device: D,
        queue: Q,
    ) -> Self {
        UI_LAUNCHED.call_once(|| {});
        let window = winit::window::Window::new(&event_loop).unwrap();

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let size = window.inner_size();

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(device.borrow(), &surface_config);

        let ctx = egui::Context::default();

        let renderer = egui_wgpu::Renderer::new(device.borrow(), swapchain_format, None, 1);
        let state = egui_winit::State::new(&window);

        Self {
            window,
            device,
            queue,
            state,
            renderer,
            size,
            ctx,
            surface,
            surface_config,
        }
    }
}
impl<W, D, Q> UiResources<W, D, Q>
where
    W: Borrow<Window>,
    D: Borrow<wgpu::Device>,
    Q: Borrow<wgpu::Queue>,
{
    pub fn handle_event<T>(
        &mut self,
        event: &winit::event::Event<'_, T>,
        control_flow: &mut ControlFlow,
    ) -> bool {
        let mut consumed_event = false;
        let device = self.device.borrow();
        let queue = self.queue.borrow();
        let window = self.window.borrow();
        let window_id = window.id();
        if let Event::WindowEvent {
            ref event,
            window_id: id,
        } = event
        {
            if window_id == *id {
                let r = self.state.on_event(&self.ctx, &event);
                consumed_event = r.consumed
            }
        }
        if let Event::RedrawRequested(id) = event {
            if *id != window_id {
                return false;
            }
        }
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                window_id,
            } if *window_id == window.id() => {
                self.size = *new_size;
                self.surface_config.width = new_size.width;
                self.surface_config.height = new_size.height;
                self.surface.configure(device, &self.surface_config);
                return true;
            }
            Event::MainEventsCleared | Event::RedrawRequested(_) => {
                self.ctx.begin_frame(self.state.take_egui_input(window));

                draw_frame(&mut self.ctx);

                let egui_output = self.ctx.end_frame();

                let tris = self.ctx.tessellate(egui_output.shapes);

                for (tex_id, delta) in egui_output.textures_delta.set {
                    self.renderer
                        .update_texture(&device, &queue, tex_id, &delta);
                }
                match self.surface.get_current_texture() {
                    Ok(frame) => {
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        let mut encoder = device.create_command_encoder(&Default::default());

                        {
                            self.renderer.update_buffers(
                                &device,
                                &queue,
                                &mut encoder,
                                &tris,
                                &egui_wgpu::renderer::ScreenDescriptor {
                                    size_in_pixels: [self.size.width, self.size.height],
                                    pixels_per_point: 1.0,
                                },
                            );

                            let mut egui_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: None,
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Load,
                                            store: true,
                                        },
                                    })],
                                    ..Default::default()
                                });

                            self.renderer.render(
                                &mut egui_pass,
                                &tris,
                                &egui_wgpu::renderer::ScreenDescriptor {
                                    size_in_pixels: [self.size.width, self.size.height],
                                    pixels_per_point: 1.0,
                                },
                            );
                        }
                        queue.submit(Some(encoder.finish()));

                        frame.present();
                    }
                    Err(e) => {
                        println!("Failed to acquire next swap chain texture {}", e);
                    }
                }
                for tex_id in egui_output.textures_delta.free {
                    self.renderer.free_texture(&tex_id);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        };
        consumed_event
    }
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
pub fn ui_thread() {
    #[cfg(target_os = "windows")]
    use egui_winit::winit::platform::windows::EventLoopBuilderExtWindows;
    #[cfg(target_os = "linux")]
    use egui_winit::winit::platform::x11::EventLoopBuilderExtX11;

    let mut event_loop = winit::event_loop::EventLoopBuilder::new()
        .with_any_thread(true)
        .build();

    let instance = wgpu::Instance::default();

    let (adapter, device, queue) = setup_wgpu(&instance, None);

    let mut ui = UiResources::new(&mut event_loop, &instance, &adapter, device, queue);

    event_loop.run(move |event, _, control_flow| {
        ui.handle_event(&event, control_flow);
    });
}

pub fn setup_wgpu(
    instance: &wgpu::Instance,
    surface: Option<&wgpu::Surface>,
) -> (wgpu::Adapter, wgpu::Device, wgpu::Queue) {
    pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: surface,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    limits: wgpu::Limits::default().using_resolution(adapter.limits()),
                    ..Default::default()
                },
                None,
            )
            .await
            .expect("Failed to create device");
        (adapter, device, queue)
    })
}

use crate::PARAMS;

fn draw_frame(ctx: &mut egui::Context) {
    egui::CentralPanel::default().show(&ctx, |ui| {
        for param in PARAMS {
            egui::CollapsingHeader::new(param.name).show(ui, |ui| {
                if let Some((any, viewer)) = param.inner.get() {
                    if viewer.is_for(&**any) {
                        viewer.draw(&**any, ui);
                    } else {
                        ui.label("Viewer doesn't support this type - this is likely a bug");
                    }
                } else {
                    ui.label("Param not initalized yet");
                }
            });
        }
    });
}
trait DrawParam: Sized {
    fn draw(&self, name: &'static str, ui: &mut egui::Ui);
}
