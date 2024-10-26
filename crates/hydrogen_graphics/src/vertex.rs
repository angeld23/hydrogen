use hydrogen_math::rect::OrientedSection;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2D {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub tex_index: u32,
    pub color: [f32; 4],
}

impl Vertex2D {
    pub const VERTEX_FORMAT: &'static [wgpu::VertexFormat] = &[
        wgpu::VertexFormat::Float32x2,
        wgpu::VertexFormat::Float32x2,
        wgpu::VertexFormat::Uint32,
        wgpu::VertexFormat::Float32x4,
    ];

    pub fn fill_screen(
        color: impl Into<[f32; 4]>,
        section: impl Into<OrientedSection>,
    ) -> [Self; 4] {
        let color = color.into();
        let section = section.into();

        let uv = section.uv_corners();
        let tex_index = section.section.layer_index;

        [
            Self {
                pos: [0.0, 0.0],
                uv: uv.top_left,
                tex_index,
                color,
            },
            Self {
                pos: [0.0, 1.0],
                uv: uv.bottom_left,
                tex_index,
                color,
            },
            Self {
                pos: [1.0, 1.0],
                uv: uv.bottom_right,
                tex_index,
                color,
            },
            Self {
                pos: [1.0, 0.0],
                uv: uv.top_right,
                tex_index,
                color,
            },
        ]
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3D {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub tex_index: u32,
    pub normal: [f32; 3],
}

impl Vertex3D {
    pub const VERTEX_FORMAT: &'static [wgpu::VertexFormat] = &[
        wgpu::VertexFormat::Float32x3,
        wgpu::VertexFormat::Float32x2,
        wgpu::VertexFormat::Uint32,
        wgpu::VertexFormat::Float32x3,
    ];
}
