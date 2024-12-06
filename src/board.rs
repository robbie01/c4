use std::{mem, num::NonZero};

use bytemuck::{Pod, Zeroable, cast_slice, cast_slice_mut};
use nalgebra::{Matrix4, Point3, Translation3};
use wgpu::{*, util::{BufferInitDescriptor, DeviceExt as _}};

const ROWS: usize = 6;
const COLS: usize = 7;

const HALF_ROWS: f32 = ROWS as f32 / 2.;
const HALF_COLS: f32 = COLS as f32 / 2.;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct BoardVertex {
    position: [f32; 3],
    coord: [f32; 2],
}

const BOARD_VERTICES: &[BoardVertex] = &[
    // Front
    BoardVertex { position: [-HALF_COLS, HALF_ROWS, 0.1], coord: [0., 0.] },  // top left
    BoardVertex { position: [-HALF_COLS, -HALF_ROWS, 0.1], coord: [0., ROWS as f32] }, // bottom left
    BoardVertex { position: [HALF_COLS, -HALF_ROWS, 0.1], coord: [COLS as f32, ROWS as f32] },  // bottom right
    BoardVertex { position: [-HALF_COLS, HALF_ROWS, 0.1], coord: [0., 0.] },  // top left
    BoardVertex { position: [HALF_COLS, -HALF_ROWS, 0.1], coord: [COLS as f32, ROWS as f32] },  // bottom right
    BoardVertex { position: [HALF_COLS, HALF_ROWS, 0.1], coord: [COLS as f32, 0.] },   // top right

    // Back
    BoardVertex { position: [-HALF_COLS, HALF_ROWS, -0.1], coord: [COLS as f32, 0.] },  // top right
    BoardVertex { position: [HALF_COLS, -HALF_ROWS, -0.1], coord: [0., ROWS as f32] },  // bottom left
    BoardVertex { position: [-HALF_COLS, -HALF_ROWS, -0.1], coord: [COLS as f32, ROWS as f32] }, // bottom right
    BoardVertex { position: [-HALF_COLS, HALF_ROWS, -0.1], coord: [COLS as f32, 0.] },  // top right
    BoardVertex { position: [HALF_COLS, HALF_ROWS, -0.1], coord: [0., 0.] },   // top left
    BoardVertex { position: [HALF_COLS, -HALF_ROWS, -0.1], coord: [0., ROWS as f32] },  // bottom left

    // Left
    BoardVertex { position: [-HALF_COLS, HALF_ROWS, -0.1], coord: [0., 0.] },   // top left
    BoardVertex { position: [-HALF_COLS, -HALF_ROWS, -0.1], coord: [0., 0.] },  // bottom left
    BoardVertex { position: [-HALF_COLS, HALF_ROWS, 0.1], coord: [0., 0.] },    // top right
    BoardVertex { position: [-HALF_COLS, HALF_ROWS, 0.1], coord: [0., 0.] },    // top right
    BoardVertex { position: [-HALF_COLS, -HALF_ROWS, -0.1], coord: [0., 0.] },  // bottom left
    BoardVertex { position: [-HALF_COLS, -HALF_ROWS, 0.1], coord: [0., 0.] },   // bottom right

    // Right
    BoardVertex { position: [HALF_COLS, HALF_ROWS, -0.1], coord: [0., 0.] },   // top right
    BoardVertex { position: [HALF_COLS, HALF_ROWS, 0.1], coord: [0., 0.] },    // top left
    BoardVertex { position: [HALF_COLS, -HALF_ROWS, -0.1], coord: [0., 0.] },  // bottom right
    BoardVertex { position: [HALF_COLS, HALF_ROWS, 0.1], coord: [0., 0.] },    // top left
    BoardVertex { position: [HALF_COLS, -HALF_ROWS, 0.1], coord: [0., 0.] },   // bottom left
    BoardVertex { position: [HALF_COLS, -HALF_ROWS, -0.1], coord: [0., 0.] },  // bottom right
];

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct TileVertex {
    position: [f32; 3],
    normal: [f32; 3]
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct TileInstance {
    model_mat: Matrix4<f32>,
    color: [f32; 4]
}

const FRAC_SQRT3_4: f32 = 0.43301270189221932338186158537646809173570131345259515701395174486298325422;

const TILE_VERTICES: &[TileVertex] = &[
    // Front hexagon
    TileVertex { position: [0.5, 0., 0.1], normal: [0., 0., 1.] },
    TileVertex { position: [0.25, FRAC_SQRT3_4, 0.1], normal: [0., 0., 1.] },
    TileVertex { position: [-0.25, FRAC_SQRT3_4, 0.1], normal: [0., 0., 1.] },
    TileVertex { position: [-0.5, 0., 0.1], normal: [0., 0., 1.] },
    TileVertex { position: [-0.25, -FRAC_SQRT3_4, 0.1], normal: [0., 0., 1.] },
    TileVertex { position: [0.25, -FRAC_SQRT3_4, 0.1], normal: [0., 0., 1.] },

    // Back hexagon
    TileVertex { position: [FRAC_SQRT3_4, 0.25, -0.1], normal: [0., 0., -1.] },
    TileVertex { position: [0., 0.5, -0.1], normal: [0., 0., -1.] },
    TileVertex { position: [-FRAC_SQRT3_4, 0.25, -0.1], normal: [0., 0., -1.] },
    TileVertex { position: [-FRAC_SQRT3_4, -0.25, -0.1], normal: [0., 0., -1.] },
    TileVertex { position: [0., -0.5, -0.1], normal: [0., 0., -1.] },
    TileVertex { position: [FRAC_SQRT3_4, -0.25, -0.1], normal: [0., 0., -1.] },

    // Front hexagon (for side triangles)
    TileVertex { position: [0.5, 0., 0.1], normal: [1., 0., 0.] },
    TileVertex { position: [0.25, FRAC_SQRT3_4, 0.1], normal: [0.5, 2.*FRAC_SQRT3_4, 0.] },
    TileVertex { position: [-0.25, FRAC_SQRT3_4, 0.1], normal: [-0.5, 2.*FRAC_SQRT3_4, 0.] },
    TileVertex { position: [-0.5, 0., 0.1], normal: [-1., 0., 0.] },
    TileVertex { position: [-0.25, -FRAC_SQRT3_4, 0.1], normal: [-0.5, -2.*FRAC_SQRT3_4, 0.] },
    TileVertex { position: [0.25, -FRAC_SQRT3_4, 0.1], normal: [0.5, -2.*FRAC_SQRT3_4, 0.] },

    // Back hexagon (for side triangles)
    TileVertex { position: [FRAC_SQRT3_4, 0.25, -0.1], normal: [2.*FRAC_SQRT3_4, 0.5, 0.] },
    TileVertex { position: [0., 0.5, -0.1], normal: [0., 1., 0.] },
    TileVertex { position: [-FRAC_SQRT3_4, 0.25, -0.1], normal: [-2.*FRAC_SQRT3_4, 0.5, 0.] },
    TileVertex { position: [-FRAC_SQRT3_4, -0.25, -0.1], normal: [-2.*FRAC_SQRT3_4, -0.5, 0.] },
    TileVertex { position: [0., -0.5, -0.1], normal: [0., -1., 0.] },
    TileVertex { position: [FRAC_SQRT3_4, -0.25, -0.1], normal: [2.*FRAC_SQRT3_4, -0.5, 0.] },
];

const TILE_INDICES: &[u16] = &[
    // Front
    0, 1, 2,
    0, 2, 3,
    0, 3, 4,
    0, 4, 5,

    // Back
    6, 8, 7,
    6, 9, 8,
    6, 10, 9,
    6, 11, 10,

    // Side triangles (hexagonal antiprism)
    12, 18, 13,
    13, 18, 19,
    13, 19, 14,
    14, 19, 20,
    14, 20, 15,
    15, 20, 21,
    15, 21, 16,
    16, 21, 22,
    16, 22, 17,
    17, 22, 23,
    17, 23, 12,
    12, 23, 18
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Red,
    Yellow
}

// TODO: coalesce buffers (all have constant size)
#[derive(Debug)]
pub struct Board {
    board_pip: RenderPipeline,
    board_vertices: Buffer,

    tile_pip: RenderPipeline,
    tile_vertices: Buffer,
    tile_indices: Buffer,
    tile_instances: Buffer,
    num_tiles: usize,

    preview: Option<u8>,
    current_player: Tile,
    win: Option<Tile>,
    tiles: [[Option<Tile>; COLS]; ROWS]
}

impl Board {
    pub fn new(dev: &Device, _q: &Queue, fmt: TextureFormat, camera_bgl: &BindGroupLayout) -> Self {
        let board_shader = dev.create_shader_module(include_wgsl!("board.wgsl"));
        let board_ppl = dev.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[camera_bgl],
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
                        attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x3],
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
        let tile_indices = dev.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: cast_slice(TILE_INDICES),
            usage: BufferUsages::INDEX
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

        Self { board_pip, board_vertices, tile_pip, tile_vertices, tile_indices, tile_instances, tiles, num_tiles: 0, preview: None, current_player: Tile::Red, win: None }
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

        if hit.x >= -HALF_COLS && hit.x <= HALF_COLS && hit.y >= -HALF_ROWS && hit.y <= HALF_ROWS {
            // Convert x position to column index (0-6)
            let col = (hit.x + HALF_COLS) as u8;
            Some(col)
        } else {
            None
        }
    }

    pub fn set_preview(&mut self, x: f32, y: f32, view_proj_inv: &Matrix4<f32>) {
        self.preview = Self::column_from_ndc(x, y, view_proj_inv);
    }

    fn find_win(&self) -> Option<Tile> {
        // Generated by Copilot

        // Check horizontal
        for row in self.tiles.iter() {
            for i in 0..=COLS-4 {
                if let Some(tile) = row[i] {
                    if row[i+1..i+4].iter().all(|&t| t == Some(tile)) {
                        return Some(tile);
                    }
                }
            }
        }

        // Check vertical
        for i in 0..=COLS-1 {
            for j in 0..=ROWS-4 {
                if let Some(tile) = self.tiles[j][i] {
                    if (0..4).all(|k| self.tiles[j+k][i] == Some(tile)) {
                        return Some(tile);
                    }
                }
            }
        }

        // Check diagonal
        for i in 0..=COLS-4 {
            for j in 0..=ROWS-4 {
                if let Some(tile) = self.tiles[j][i] {
                    if (0..4).all(|k| self.tiles[j+k][i+k] == Some(tile)) {
                        return Some(tile);
                    }
                }
            }
        }

        for i in 0..=COLS-4 {
            for j in 3..=ROWS-1 {
                if let Some(tile) = self.tiles[j][i] {
                    if (0..4).all(|k| self.tiles[j-k][i+k] == Some(tile)) {
                        return Some(tile);
                    }
                }
            }
        }

        None
    }

    pub fn drop_tile(&mut self, x: f32, y: f32, view_proj_inv: &Matrix4<f32>) {
        if self.win.is_some() {
            return;
        }

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

        if let Some(win) = self.find_win() {
            self.win = Some(win);
            println!("{:?} wins!", win);
            return;
        }
    }

    pub fn prepare(&mut self, q: &Queue) {
        let mut inst_buf = q.write_buffer_with(&self.tile_instances, 0, NonZero::new(self.tile_instances.size()).unwrap()).unwrap();
        let instances = cast_slice_mut(&mut inst_buf);

        let mut inst = 0;

        if let Some(preview) = self.preview {
            let model_mat = Translation3::new(
                preview as f32 - HALF_COLS + 0.5,
                HALF_ROWS + 1.,
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
                        j as f32 - HALF_COLS + 0.5,
                        HALF_ROWS - 0.5 - i as f32,
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
        rpass.set_index_buffer(self.tile_indices.slice(..), IndexFormat::Uint16);
        rpass.set_vertex_buffer(1, self.tile_instances.slice(..));
        rpass.set_bind_group(0, camera_bg, &[]);
        rpass.draw_indexed(0..TILE_INDICES.len() as u32, 0, 0..self.num_tiles as u32);
        
        rpass.set_pipeline(&self.board_pip);
        rpass.set_vertex_buffer(0, self.board_vertices.slice(..));
        rpass.set_bind_group(0, camera_bg, &[]);
        rpass.draw(0..BOARD_VERTICES.len() as u32, 0..1);
    }
}