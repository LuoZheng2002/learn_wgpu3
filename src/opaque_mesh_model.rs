


pub struct OpaqueMeshModel{
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub texture_bind_group: wgpu::BindGroup,
}

impl OpaqueMeshModel{
    pub fn new() -> Self{
        todo!()
    }
}