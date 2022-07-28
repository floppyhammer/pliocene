extern crate lyon;

use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::*;
use wgpu::util::DeviceExt;
use crate::Vertex;

pub struct VectorSprite {
    pub path: Path,
    geometry: VertexBuffers<MyVertex, u16>,
    mesh: VectorMesh,
}

// Let's use our own custom vertex type instead of the default one.
#[derive(Copy, Clone, Debug)]
struct MyVertex {
    position: [f32; 2],
}

impl VectorSprite {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> VectorSprite {
        // Build a Path.
        let mut builder = Path::builder();
        builder.begin(point(0.0, 0.0));
        builder.line_to(point(1.0, 0.0));
        builder.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0));
        builder.cubic_bezier_to(point(1.0, 1.0), point(0.0, 1.0), point(0.0, 0.0));
        builder.end(true);
        let path = builder.build();

        // Will contain the result of the tessellation.
        let mut geometry: VertexBuffers<MyVertex, u16> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();
        {
            // Compute the tessellation.
            tessellator.tessellate_path(
                &path,
                &FillOptions::default(),
                &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                    MyVertex {
                        position: vertex.position().to_array(),
                    }
                }),
            ).unwrap();
        }
        // The tessellated geometry is ready to be uploaded to the GPU.
        println!(" -- {} vertices {} indices",
                 geometry.vertices.len(),
                 geometry.indices.len()
        );

        let mut vertices = Vec::new();
        for v in &geometry.vertices {
            vertices.push(VectorVertex {
                position: [v.position[0], v.position[1]],
                color: [1.0, 1.0, 1.0],
            });
        }

        let mut indices = Vec::new();
        for i in &geometry.indices {
            indices.push(*i as i32);
        }

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Default 2D Mesh Vertex Buffer")),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Default 2D Mesh Index Buffer")),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let mesh = VectorMesh {
            name: "".to_string(),
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        };

        Self { path, geometry, mesh }
    }
}

pub trait DrawVector<'a> {
    fn draw_path(
        &mut self,
        mesh: &'a VectorMesh,
        camera_bind_group: &'a wgpu::BindGroup,
    );
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct VectorVertex {
    pub(crate) position: [f32; 2],
    pub(crate) color: [f32; 3],
}

impl Vertex for VectorVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<VectorVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { // Position.
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute { // Color.
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct VectorMesh {
    // Mesh name for debugging reason.
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl<'a, 'b> DrawVector<'b> for wgpu::RenderPass<'a>
    where
        'b: 'a,
{
    fn draw_path(
        &mut self,
        mesh: &'b VectorMesh,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        // Set vertex buffer for VertexInput.
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));

        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        // Set camera uniform.
        self.set_bind_group(1, camera_bind_group, &[]);

        self.draw_indexed(0..mesh.index_count, 0, 0..1);
    }
}