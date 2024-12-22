use std::{mem, num::NonZero};

use bytemuck::{cast_slice_mut, Pod, Zeroable};
use nalgebra::Point3;
use util::{DeviceExt, TextureDataOrder};
use wgpu::*;

use crate::camera::Camera;

const PX_PNG: &[u8] = include_bytes!("cubemap/px.png");
const NX_PNG: &[u8] = include_bytes!("cubemap/nx.png");
const PY_PNG: &[u8] = include_bytes!("cubemap/py.png");
const NY_PNG: &[u8] = include_bytes!("cubemap/ny.png");
const PZ_PNG: &[u8] = include_bytes!("cubemap/pz.png");
const NZ_PNG: &[u8] = include_bytes!("cubemap/nz.png");

fn png_to_bytes(mut png: &[u8]) -> (u32, u32, Vec<u8>) {
    let mut decoder = png::Decoder::new(&mut png);
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut buf).unwrap();
    let info = reader.info();
    (info.width, info.height, buf)
}

#[derive(Debug)]
pub struct Skybox {
    vertices: Buffer,
    pip: RenderPipeline,
    bg: BindGroup
}

const SKYBOX_VERTICES: &[[f32; 2]] = &[
    [-1., -1.],
    [3., -1.],
    [-1., 3.]
];

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct SkyboxVertex {
    pos: [f32; 2],
    coord: Point3<f32>
}

impl Skybox {
    pub fn new(dev: &Device, q: &Queue, fmt: TextureFormat) -> Self {
        // Double entendre: "px" for "pixel" and "positive x"
        let (px_w, px_h, mut px) = png_to_bytes(PX_PNG);
        let (nx_w, nx_h, mut nx) = png_to_bytes(NX_PNG);
        let (py_w, py_h, mut py) = png_to_bytes(PY_PNG);
        let (ny_w, ny_h, mut ny) = png_to_bytes(NY_PNG);
        let (pz_w, pz_h, mut pz) = png_to_bytes(PZ_PNG);
        let (nz_w, nz_h, mut nz) = png_to_bytes(NZ_PNG);

        assert!(px_w == nx_w && nx_w == py_w && py_w == ny_w && ny_w == pz_w && pz_w == nz_w);
        assert!(px_h == nx_h && nx_h == py_h && py_h == ny_h && ny_h == pz_h && pz_h == nz_h);

        px.append(&mut nx);
        px.append(&mut py);
        px.append(&mut ny);
        px.append(&mut pz);
        px.append(&mut nz);

        let tex = dev.create_texture_with_data(&q, &TextureDescriptor {
            label: None,
            size: Extent3d {
                width: px_w,
                height: px_h,
                depth_or_array_layers: 6
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
        }, TextureDataOrder::LayerMajor, &px);

        let vertices = dev.create_buffer(&BufferDescriptor {
            label: None,
            size: (mem::size_of::<SkyboxVertex>()*SKYBOX_VERTICES.len()) as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false
        });

        let bgl = dev.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::Cube,
                        multisampled: false
                    },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None
                }
            ]
        });

        let shader = dev.create_shader_module(include_wgsl!("skybox.wgsl"));
        let ppl = dev.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[]
        });

        let pip = dev.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&ppl),
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                buffers: &[VertexBufferLayout {
                    array_stride: mem::size_of::<SkyboxVertex>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: mem::size_of::<[f32; 2]>() as u64,
                            shader_location: 1
                        }
                    ]
                }],
                compilation_options: Default::default()
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: None,
                targets: &[Some(ColorTargetState {
                    format: fmt,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default()
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Always,
                stencil: Default::default(),
                bias: Default::default()
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            multiview: None,
            cache: None
        });

        let bg = dev.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bgl,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&tex.create_view(&TextureViewDescriptor {
                        dimension: Some(TextureViewDimension::Cube),
                        ..Default::default()
                    }))
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&dev.create_sampler(&SamplerDescriptor {
                        label: None,
                        address_mode_u: AddressMode::ClampToEdge,
                        address_mode_v: AddressMode::ClampToEdge,
                        address_mode_w: AddressMode::ClampToEdge,
                        mag_filter: FilterMode::Linear,
                        min_filter: FilterMode::Linear,
                        ..Default::default()
                    }))
                }
            ]
        });

        Self {
            vertices,
            pip, bg
        }
    }

    pub fn prepare(&mut self, q: &Queue, cam: &mut Camera) {
        let mut view = q.write_buffer_with(&self.vertices, 0, NonZero::new(self.vertices.size()).unwrap()).unwrap();
        for (v, vert) in SKYBOX_VERTICES.iter().zip(cast_slice_mut::<_, SkyboxVertex>(&mut view)) {
            vert.pos = *v;
            vert.coord = cam.unproject_point(&Point3::new(v[0], v[1], 1.));
        }
    }

    pub fn render<'rpass>(&'rpass self, rpass: &mut RenderPass<'rpass>) {
        rpass.set_pipeline(&self.pip);
        rpass.set_bind_group(0, &self.bg, &[]);
        rpass.set_vertex_buffer(0, self.vertices.slice(..));
        rpass.draw(0..SKYBOX_VERTICES.len() as u32, 0..1);
    }
}