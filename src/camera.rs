use std::{f32::consts::PI, mem, num::NonZeroU64};

use bytemuck::from_bytes_mut;
use nalgebra::{Isometry3, Matrix4, Perspective3, Point3, UnitQuaternion, UnitVector3, Vector3};
use wgpu::*;

#[derive(Debug)]
pub struct Camera {
    eye: Point3<f32>,
    angle: f32,
    target: Point3<f32>,
    up: UnitVector3<f32>,
    proj: Perspective3<f32>,
    buf: Buffer,
    bgl: BindGroupLayout,
    bg: BindGroup
}

impl Camera {
    pub fn new(dev: &Device, aspect: f32) -> Self {
        let buf = dev.create_buffer(&BufferDescriptor {
            label: None,
            size: mem::size_of::<Matrix4<f32>>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let bgl = dev.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                }
            ]
        });

        let bg = dev.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bgl,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buf.as_entire_binding()
            }]
        });

        Self {
            eye: Point3::new(0., 0., 16.),
            angle: 1.,
            target: Point3::origin(),
            up: UnitVector3::new_unchecked(Vector3::y()),
            proj: Perspective3::new(aspect, 45. * PI / 180., 0.1, 100.),
            buf, bgl, bg
        }
    }

    pub fn add_angle(&mut self, angle: f32) {
        self.angle = (self.angle + angle) % (2. * PI);
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.proj.set_aspect(aspect);
    }

    // TODO: cache this
    pub fn view_proj(&self) -> Matrix4<f32> {
        let rot = UnitQuaternion::from_axis_angle(&self.up, self.angle);
        let eye = Isometry3::rotation_wrt_point(rot, self.target) * &self.eye;
        let view = Matrix4::look_at_rh(&eye, &self.target, &self.up);

        self.proj.as_matrix() * view
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bgl
    }

    pub fn bind_group<'a>(&self, q: &'a Queue) -> &BindGroup {
        let mut view = q.write_buffer_with(&self.buf, 0, NonZeroU64::new(self.buf.size()).unwrap()).unwrap();
        *from_bytes_mut(&mut view) = self.view_proj();
        &self.bg
    }
}