// use anyhow::Result;
// use futures::executor::block_on;
// use std::{sync::Arc, vec};
// use wgpu::{CommandEncoderDescriptor, PipelineLayoutDescriptor, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, ShaderModuleDescriptor, TextureViewDescriptor};
// use wgpu::{
//     self, wgt::DeviceDescriptor, Device, Instance, Queue, RequestAdapterOptions, Surface,
//     SurfaceConfiguration, Texture, TextureDescriptor, TextureUsages,
// };
// use winit::window::Window;

// pub struct Render {
//     pub device: Device,
//     pub queue: Queue,
//     pub texture: Texture,
//     pub surface: Surface<'static>,
//     pub config: SurfaceConfiguration,
//     pub is_surface_configured: bool,
//     pub render_pipeline: RenderPipeline,
//     pub window: Arc<Window>,
// }

// impl Render {
//     pub fn new(window: Arc<Window>) -> Result<Render> {
//         let instance = Instance::default();

//         let surface = instance.create_surface(window.clone())?;

//         let adapter = block_on(async {
//             instance
//                 .request_adapter(&RequestAdapterOptions {
//                     compatible_surface: Some(&surface),
//                     ..Default::default()
//                 })
//                 .await
//         })?;

//         let (device, queue) =
//             block_on(async { adapter.request_device(&DeviceDescriptor::default()).await })?;

//         let size = window.inner_size();
//         let config = SurfaceConfiguration {
//             usage: TextureUsages::RENDER_ATTACHMENT,
//             format: surface.get_capabilities(&adapter).formats[0],
//             width: size.width,
//             height: size.height,
//             present_mode: wgpu::PresentMode::Fifo,
//             alpha_mode: wgpu::CompositeAlphaMode::Auto,
//             view_formats: vec![],
//             desired_maximum_frame_latency: 0,
//         };
//         surface.configure(&device, &config);

//         let texture = device.create_texture(&TextureDescriptor {
//             label: Some("Emulator Framebuffer"),
//             size: wgpu::Extent3d {
//                 width: 160,
//                 height: 144,
//                 depth_or_array_layers: 1,
//             },
//             mip_level_count: 1,
//             sample_count: 1,
//             dimension: wgpu::TextureDimension::D2,
//             format: wgpu::TextureFormat::Rgba8UnormSrgb,
//             usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
//             view_formats: &[],
//         });

//         let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

//         let render_pipeline =
//             device.create_pipeline_layout(&PipelineLayoutDescriptor {
//                 label: Some("Render Pipeline Layout"),
//                 bind_group_layouts: &[],
//                 push_constant_ranges: &[],
//             });

//         Ok(Render {
//             device,
//             queue,
//             texture,
//             surface,
//             config,
//             is_surface_configured: false,
//             window,
//         })
//     }

//     pub fn draw(&mut self, pixels: &[u8]) {
//         let frame = match self.surface.get_current_texture() {
//             Ok(frame) => frame,
//             Err(_) => {
//                 self.surface.configure(&self.device, &self.config);
//                 self
//                     .surface
//                     .get_current_texture()
//                     .expect("Failed to acquire surface texture after reconfiguration")
//             }
//         };

//         let view = frame.texture.create_view(&TextureViewDescriptor::default());
//         let mut encoder =
//         self
//                 .device
//                 .create_command_encoder(&CommandEncoderDescriptor {
//                     label: Some("Render Encoder"),
//         });
//         {
//             let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
//                 label: Some("Render Pass"),
//                 color_attachments: &[Some(RenderPassColorAttachment {
//                     view: &view,
//                     resolve_target: None,
//                     ops: wgpu::Operations {
//                         load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
//                         store: wgpu::StoreOp::Store,
//                     },
//                     depth_slice: None,
//                 })],
//                 depth_stencil_attachment: None,
//                 timestamp_writes: None,
//                 occlusion_query_set: None,
//             });
//             // draw
//         }
//         self.queue.submit(Some(encoder.finish()));
//         frame.present();
//     }
// }
