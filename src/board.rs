use std::{mem, num::NonZero};

use bytemuck::{Pod, Zeroable, cast_slice, cast_slice_mut};
use nalgebra::{Matrix4, Point3, Translation3};
use util::{BufferInitDescriptor, DeviceExt, TextureDataOrder};
use wgpu::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct BoardVertex {
    position: [f32; 3],
    coord: [f32; 2],
}

const BOARD_VERTICES: &[BoardVertex] = &[
    // Front
    BoardVertex { position: [-3.5, 3., 0.1], coord: [0.0, 0.0] },  // top left
    BoardVertex { position: [-3.5, -3., 0.1], coord: [0.0, 1.0] }, // bottom left
    BoardVertex { position: [3.5, -3., 0.1], coord: [1.0, 1.0] },  // bottom right
    BoardVertex { position: [-3.5, 3., 0.1], coord: [0.0, 0.0] },  // top left
    BoardVertex { position: [3.5, -3., 0.1], coord: [1.0, 1.0] },  // bottom right
    BoardVertex { position: [3.5, 3., 0.1], coord: [1.0, 0.0] },   // top right

    // Back
    BoardVertex { position: [-3.5, 3., -0.1], coord: [0.0, 0.0] },  // top right
    BoardVertex { position: [3.5, -3., -0.1], coord: [1.0, 1.0] },  // bottom left
    BoardVertex { position: [-3.5, -3., -0.1], coord: [0.0, 1.0] }, // bottom right
    BoardVertex { position: [-3.5, 3., -0.1], coord: [0.0, 0.0] },  // top right
    BoardVertex { position: [3.5, 3., -0.1], coord: [1.0, 0.0] },   // top left
    BoardVertex { position: [3.5, -3., -0.1], coord: [1.0, 1.0] },  // bottom left

    // Left
    BoardVertex { position: [-3.5, 3., -0.1], coord: [0.0, 0.0] },   // top left
    BoardVertex { position: [-3.5, -3., -0.1], coord: [0.0, 0.0] },  // bottom left
    BoardVertex { position: [-3.5, 3., 0.1], coord: [0.0, 0.0] },    // top right
    BoardVertex { position: [-3.5, 3., 0.1], coord: [0.0, 0.0] },    // top right
    BoardVertex { position: [-3.5, -3., -0.1], coord: [0.0, 0.0] },  // bottom left
    BoardVertex { position: [-3.5, -3., 0.1], coord: [0.0, 0.0] },   // bottom right

    // Right
    BoardVertex { position: [3.5, 3., -0.1], coord: [0.0, 0.0] },   // top right
    BoardVertex { position: [3.5, 3., 0.1], coord: [0.0, 0.0] },    // top left
    BoardVertex { position: [3.5, -3., -0.1], coord: [0.0, 0.0] },  // bottom right
    BoardVertex { position: [3.5, 3., 0.1], coord: [0.0, 0.0] },    // top left
    BoardVertex { position: [3.5, -3., 0.1], coord: [0.0, 0.0] },   // bottom left
    BoardVertex { position: [3.5, -3., -0.1], coord: [0.0, 0.0] },  // bottom right
];

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct TileVertex {
    position: [f32; 3]
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct TileInstance {
    model_mat: Matrix4<f32>,
    color: [f32; 4]
}

const TILE_VERTICES: &[TileVertex] = &[
    // Front
    TileVertex { position: [-0.5, 0.5, 0.09], },  // top left
    TileVertex { position: [-0.5, -0.5, 0.09], }, // bottom left
    TileVertex { position: [0.5, -0.5, 0.09], },  // bottom right
    TileVertex { position: [-0.5, 0.5, 0.09], },  // top left
    TileVertex { position: [0.5, -0.5, 0.09], },  // bottom right
    TileVertex { position: [0.5, 0.5, 0.09], },   // top right

    // Back
    TileVertex { position: [-0.5, 0.5, -0.09] },  // top right
    TileVertex { position: [0.5, -0.5, -0.09] },  // bottom left
    TileVertex { position: [-0.5, -0.5, -0.09] }, // bottom right
    TileVertex { position: [-0.5, 0.5, -0.09] },  // top right
    TileVertex { position: [0.5, 0.5, -0.09] },   // top left
    TileVertex { position: [0.5, -0.5, -0.09] },  // bottom left

    // Left
    TileVertex { position: [-0.5, 0.5, -0.09] },   // top left
    TileVertex { position: [-0.5, -0.5, -0.09] },  // bottom left
    TileVertex { position: [-0.5, 0.5, 0.09] },    // top right
    TileVertex { position: [-0.5, 0.5, 0.09] },    // top right
    TileVertex { position: [-0.5, -0.5, -0.09] },  // bottom left
    TileVertex { position: [-0.5, -0.5, 0.09] },   // bottom right

    // Right
    TileVertex { position: [0.5, 0.5, -0.09] },   // top right
    TileVertex { position: [0.5, 0.5, 0.09] },    // top left
    TileVertex { position: [0.5, -0.5, -0.09] },  // bottom right
    TileVertex { position: [0.5, 0.5, 0.09] },    // top left
    TileVertex { position: [0.5, -0.5, 0.09] },   // bottom left
    TileVertex { position: [0.5, -0.5, -0.09] },  // bottom right

    // Bottom (y = -0.5)
    TileVertex { position: [-0.5, -0.5, -0.09] },  // bottom left
    TileVertex { position: [0.5, -0.5, -0.09] },   // bottom right
    TileVertex { position: [0.5, -0.5, 0.09] },    // top right
    TileVertex { position: [-0.5, -0.5, -0.09] },  // bottom left
    TileVertex { position: [0.5, -0.5, 0.09] },    // top right
    TileVertex { position: [-0.5, -0.5, 0.09] },   // top left
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Red,
    Yellow
}

const ROWS: usize = 6;
const COLS: usize = 7;

// TODO: coalesce buffers (all have constant size)
#[derive(Debug)]
pub struct Board {
    board_tex_bg: BindGroup,
    board_pip: RenderPipeline,
    board_vertices: Buffer,

    tile_pip: RenderPipeline,
    tile_vertices: Buffer,
    tile_instances: Buffer,
    num_tiles: usize,

    preview: Option<u8>,
    current_player: Tile,
    tiles: [[Option<Tile>; COLS]; ROWS]
}

impl Board {
    pub fn new(dev: &Device, q: &Queue, fmt: TextureFormat, camera_bgl: &BindGroupLayout) -> Self {
        const C4: &[u8] = include_bytes!("connect4.png");

        let mut c4 = C4;
        let mut re = png::Decoder::new(&mut c4).read_info().unwrap();
        let mut data = [0u8; 4*700*600];
        re.next_frame(&mut data).unwrap();
        let i = re.info();

        let board_tex = dev.create_texture_with_data(
            q,
            &TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: i.width,
                    height: i.height,
                    depth_or_array_layers: 1
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[]
            },
            TextureDataOrder::LayerMajor,
            &data
        );

        let board_tex_bgl = dev.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false
                },
                count: None
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None
            }]
        });

        let board_tex_bg = dev.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &board_tex_bgl,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&board_tex.create_view(&TextureViewDescriptor::default()))
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&dev.create_sampler(&SamplerDescriptor {
                    address_mode_u: AddressMode::ClampToEdge,
                    address_mode_v: AddressMode::ClampToEdge,
                    address_mode_w: AddressMode::ClampToEdge,
                    mag_filter: FilterMode::Linear,
                    min_filter: FilterMode::Linear,
                    mipmap_filter: FilterMode::Nearest,
                    ..Default::default()
                }))
            }]
        });

        let board_shader = dev.create_shader_module(include_wgsl!("board.wgsl"));
        let board_ppl = dev.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[camera_bgl, &board_tex_bgl],
            push_constant_ranges: &[]
        });
        let board_pip = dev.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&board_ppl),
            vertex: VertexState {
                module: &board_shader,
                entry_point: None,
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<BoardVertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x2],
                }],
                compilation_options: PipelineCompilationOptions::default()
            },
            fragment: Some(FragmentState {
                module: &board_shader,
                entry_point: None,
                targets: &[Some(ColorTargetState {
                    format: fmt,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default()
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                ..Default::default()
            },
            // Always draw on top of everything
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Always,
                stencil: Default::default(),
                bias: Default::default()
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None
        });

        let board_vertices = dev.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: cast_slice(BOARD_VERTICES),
            usage: BufferUsages::VERTEX
        });

        let tile_shader = dev.create_shader_module(include_wgsl!("tile.wgsl"));
        let tile_ppl = dev.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[camera_bgl],
            push_constant_ranges: &[]
        });
        let tile_pip = dev.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&tile_ppl),
            vertex: VertexState {
                module: &tile_shader,
                entry_point: None,
                buffers: &[
                    // Vertex
                    VertexBufferLayout {
                        array_stride: std::mem::size_of::<TileVertex>() as BufferAddress,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &vertex_attr_array![0 => Float32x3],
                    },
                    // Instance
                    VertexBufferLayout {
                        array_stride: std::mem::size_of::<TileInstance>() as BufferAddress,
                        step_mode: VertexStepMode::Instance,
                        attributes: &vertex_attr_array![
                            10 => Float32x4,
                            11 => Float32x4,
                            12 => Float32x4,
                            13 => Float32x4,
                            14 => Float32x4
                        ],
                    }
                ],
                compilation_options: PipelineCompilationOptions::default()
            },
            fragment: Some(FragmentState {
                module: &tile_shader,
                entry_point: None,
                targets: &[Some(ColorTargetState {
                    format: fmt,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default()
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
                depth_compare: CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default()
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None
        });

        let tile_vertices = dev.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: cast_slice(TILE_VERTICES),
            usage: BufferUsages::VERTEX
        });

        let tile_instances = dev.create_buffer(&BufferDescriptor {
            label: None,
            size: (mem::size_of::<TileInstance>()*(ROWS*COLS+1)) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let tiles = Default::default();

        // tiles[0][0] = Some(Tile::Red);
        // tiles[1][2] = Some(Tile::Yellow);

        Self { board_tex_bg, board_pip, board_vertices, tile_pip, tile_vertices, tile_instances, tiles, num_tiles: 0, preview: None, current_player: Tile::Red }
    }

    fn column_from_ndc(x: f32, y: f32, view_proj_inv: &Matrix4<f32>) -> Option<u8> {
        // X and Y are NDC
        // Sets self.preview to Some(0..7) if the mouse is over the board and None otherwise
        // Uses raycasting to intersect with z=0 plane

        let near = view_proj_inv.transform_point(&Point3::new(x, y, -1.));
        let far = view_proj_inv.transform_point(&Point3::new(x, y, 1.));

        // Find point collinear to near and far such that z=0
        let dir = far - near;
        let t = -near.z / dir.z;
        let hit = near + dir * t;

        if hit.x >= -3.5 && hit.x <= 3.5 && hit.y >= -3.0 && hit.y <= 3.0 {
            // Convert x position to column index (0-6)
            let col = ((hit.x + 3.5) / 7.0 * COLS as f32) as u8;
            Some(col)
        } else {
            None
        }
    }

    pub fn set_preview(&mut self, x: f32, y: f32, view_proj_inv: &Matrix4<f32>) {
        self.preview = Self::column_from_ndc(x, y, view_proj_inv);
    }

    pub fn drop_tile(&mut self, x: f32, y: f32, view_proj_inv: &Matrix4<f32>) {
        if let Some(col) = Self::column_from_ndc(x, y, view_proj_inv) {
            let col = col as usize;
            for row in self.tiles.iter_mut().rev() {
                if row[col].is_none() {
                    row[col] = Some(self.current_player);
                    self.current_player = match self.current_player {
                        Tile::Red => Tile::Yellow,
                        Tile::Yellow => Tile::Red
                    };
                    break;
                }
            }
        }
    }

    pub fn prepare(&mut self, q: &Queue) {
        let mut inst_buf = q.write_buffer_with(&self.tile_instances, 0, NonZero::new(self.tile_instances.size()).unwrap()).unwrap();
        let instances = cast_slice_mut(&mut inst_buf);

        let mut inst = 0;

        if let Some(preview) = self.preview {
            let model_mat = Translation3::new(
                preview as f32 - 3.,
                4.,
                0.
            ).to_homogeneous();
            instances[inst] = TileInstance {
                model_mat,
                color: [0.5, 0.5, 0.5, 1.]
            };
            inst += 1;
        }

        for (i, row) in self.tiles.iter().enumerate() {
            for (j, tile) in row.iter().enumerate() {
                if let Some(tile) = tile {
                    let model_mat = Translation3::new(
                        j as f32 - 3.,
                        2.5 - i as f32,
                        0.
                    ).to_homogeneous();
                    instances[inst] = TileInstance {
                        model_mat,
                        color: match tile {
                            Tile::Red => [1., 0., 0., 1.],
                            Tile::Yellow => [1., 1., 0., 1.]
                        }
                    };
                    inst += 1;
                }
            }
        }

        self.num_tiles = inst;
    }

    pub fn render<'rpass>(&'rpass self, rpass: &mut RenderPass<'rpass>, camera_bg: &'rpass BindGroup) {
        rpass.set_pipeline(&self.tile_pip);
        rpass.set_vertex_buffer(0, self.tile_vertices.slice(..));
        rpass.set_vertex_buffer(1, self.tile_instances.slice(..));
        rpass.set_bind_group(0, camera_bg, &[]);
        rpass.draw(0..TILE_VERTICES.len() as u32, 0..self.num_tiles as u32);
        
        rpass.set_pipeline(&self.board_pip);
        rpass.set_vertex_buffer(0, self.board_vertices.slice(..));
        rpass.set_bind_group(0, camera_bg, &[]);
        rpass.set_bind_group(1, &self.board_tex_bg, &[]);
        rpass.draw(0..BOARD_VERTICES.len() as u32, 0..1);
    }
}