use std::{iter, sync::Arc};

use pollster::FutureExt;
use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{board::Board, camera::Camera};

#[derive(Debug)]
pub struct State {
    win: Arc<Window>,
    sfc: Surface<'static>,
    cfg: SurfaceConfiguration,
    dev: Device,
    q: Queue,
    cam: Camera,
    bd: Board,
    pub horiz_right: bool,
    pub horiz_left: bool
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

        let cam = Camera::new(&dev, aspect);
        let bd = Board::new(&dev, &q, cfg.format, cam.bind_group_layout());

        Self {
            win, sfc, dev, q, cam, cfg, bd,
            horiz_right: false,
            horiz_left: false
        }
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
        let angle_mag = 0.02;
        let angle_delta = angle_mag * self.horiz_right as i8 as f32 - angle_mag * self.horiz_left as i8 as f32;
        self.cam.add_angle(angle_delta);

        let camerabg = self.cam.bind_group(&self.q);
        self.bd.prepare(&self.q);

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
        self.bd.render(&mut rpass, camerabg);
        drop(rpass);

        self.q.submit(iter::once(enc.finish()));

        tex.present();
    }
}