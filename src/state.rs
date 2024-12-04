use std::{iter, sync::Arc};

use pollster::FutureExt;
use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{board::Vertex, camera::Camera};

#[derive(Debug)]
pub struct State {
    win: Arc<Window>,
    sfc: Surface<'static>,
    cfg: SurfaceConfiguration,
    dev: Device,
    q: Queue,
    pip: RenderPipeline,
    cam: Camera
}

impl State {
    pub fn new(win: Window) -> Self {
        let win = Arc::new(win);
        let sz = win.inner_size();

        let inst = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let sfc = inst.create_surface(win.clone()).unwrap();
        let adpt = inst.request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&sfc),
            power_preference: PowerPreference::HighPerformance,
            ..Default::default()
        }).block_on().unwrap();

        let mut cfg = sfc.get_default_config(&adpt, sz.width, sz.height).unwrap();
        cfg.present_mode = PresentMode::Fifo;
        let (dev, q) = adpt.request_device(&Default::default(), None).block_on().unwrap();
        sfc.configure(&dev, &cfg);

        let aspect = sz.width as f32 / sz.height as f32;

        let shader = dev.create_shader_module(include_wgsl!("shader.wgsl"));
        let layout = dev.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[]
        });
        let pip = dev.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x3],
                }],
                compilation_options: PipelineCompilationOptions::default()
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: None,
                targets: &[Some(ColorTargetState {
                    format: cfg.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default()
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            multiview: None,
            cache: None
        });

        Self { win, sfc, dev, q, cam: Camera::new(aspect), cfg, pip }
    }

    pub fn win(&self) -> &Window {
        &self.win
    }

    pub fn resize(&mut self, sz: PhysicalSize<u32>) {
        let aspect = sz.width as f32 / sz.height as f32;
        self.cam.set_aspect(aspect);

        self.cfg.width = sz.width;
        self.cfg.height = sz.height;
        self.sfc.configure(&self.dev, &self.cfg);
    }

    pub fn render(&mut self) {
        let tex = self.sfc.get_current_texture().unwrap();

        let view = tex.texture.create_view(&Default::default());

        let mut enc = self.dev.create_command_encoder(&Default::default());
        let mut rpass = enc.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    // Cornflower blue
                    load: LoadOp::Clear(Color {
                        r: 0.127438,
                        g: 0.300544,
                        b: 0.846873,
                        a: 1.
                    }),
                    store: StoreOp::Store
                }
            })],
            ..Default::default()
        });
        drop(rpass);

        self.q.submit(iter::once(enc.finish()));

        tex.present();
    }
}