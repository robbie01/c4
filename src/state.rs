use std::{iter, sync::Arc};

use pollster::FutureExt;
use wgpu::*;
use winit::{dpi::{PhysicalPosition, PhysicalSize}, window::Window};

use crate::{board::Board, camera::Camera, skybox::Skybox};

#[derive(Debug)]
pub struct State {
    win: Arc<Window>,
    sfc: Surface<'static>,
    cfg: SurfaceConfiguration,
    depth_cfg: TextureDescriptor<'static>,
    depth: Texture,
    depth_view: TextureView,
    dev: Device,
    q: Queue,
    sky: Skybox,
    cam: Camera,
    bd: Board,
    last_mouse: Option<PhysicalPosition<f64>>,
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
            })
            .block_on()
            .unwrap();

        let caps = sfc.get_capabilities(&adpt);
        println!("{caps:?}");

        let mut cfg = sfc.get_default_config(&adpt, sz.width, sz.height).unwrap();
        cfg.present_mode = PresentMode::Fifo;
        if caps.formats.contains(&TextureFormat::Rgba16Float) {
            println!("HDR supported");
            cfg.format = TextureFormat::Rgba16Float;
        }
        let (dev, q) = adpt
            .request_device(&Default::default(), None)
            .block_on()
            .unwrap();
        sfc.configure(&dev, &cfg);

        let depth_cfg = TextureDescriptor {
            label: None,
            size: Extent3d {
                width: cfg.width,
                height: cfg.height,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
        };

        let depth = dev.create_texture(&depth_cfg);
        let depth_view = depth.create_view(&Default::default());

        let aspect = sz.width as f32 / sz.height as f32;

        let sky = Skybox::new(&dev, &q, cfg.format);
        let cam = Camera::new(&dev, aspect);
        let bd = Board::new(&dev, &q, cfg.format, cam.bind_group_layout());

        win.set_visible(true);

        Self {
            win, sfc, dev, q, sky, cam, cfg, bd, depth_cfg, depth, depth_view,
            horiz_right: false,
            horiz_left: false,
            last_mouse: None
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

        self.depth_cfg.size.width = sz.width;
        self.depth_cfg.size.height = sz.height;
        self.depth = self.dev.create_texture(&self.depth_cfg);
        self.depth_view = self.depth.create_view(&Default::default());
        
        if let Some(pos) = self.last_mouse {
            self.mouse_move(pos);
        }
    }

    fn update_preview(&mut self) {
        if let Some(pos) = self.last_mouse {
            let sz = self.win.inner_size();
            let x = pos.x as f32 / sz.width as f32 * 2. - 1.;
            let y = 1. - pos.y as f32 / sz.height as f32 * 2.;
            self.bd.set_preview(x, y, &mut self.cam);
        }
    }

    pub fn mouse_move(&mut self, pos: PhysicalPosition<f64>) {
        self.last_mouse = Some(pos);
    }

    pub fn mouse_click(&mut self) {
        self.update_preview();
        self.bd.drop_tile();
    }

    pub fn render(&mut self) {
        let angle_mag = 0.02;
        let angle_delta = angle_mag * self.horiz_right as i8 as f32 - angle_mag * self.horiz_left as i8 as f32;
        self.cam.add_angle(angle_delta);

        self.update_preview();

        self.sky.prepare(&self.q, &mut self.cam);
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
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.),
                    store: StoreOp::Store
                }),
                stencil_ops: None
            }),
            ..Default::default()
        });
        self.sky.render(&mut rpass);
        self.bd.render(&mut rpass, camerabg);
        drop(rpass);

        self.q.submit(iter::once(enc.finish()));

        tex.present();
    }
}