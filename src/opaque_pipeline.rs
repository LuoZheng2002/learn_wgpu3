

// it contains a RenderPipeline, can store drawables, and render them
// different pipelines have different binding requirements, so the model types are different

use std::{collections::HashMap, sync::Arc};

use russimp::{camera, mesh};
use wgpu::{core::device, util::DeviceExt, RenderPipeline};

use crate::{cache::{CacheKey, CacheValue, CACHE}, mesh_meta::MeshMeta, my_texture::MyTexture, opaque_mesh_instance::OpaqueMeshInstance, opaque_mesh_info::OpaqueMeshInfo, vertex::Vertex};

// model
// mesh_num
// opauqe mesh, transparent mesh
pub struct OpaquePipeline{
    pipeline: RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl OpaquePipeline{

    fn create_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    fn create_pipeline(
        device: &wgpu::Device, 
        config: &wgpu::SurfaceConfiguration,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("opaque.wgsl").into()),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[Vertex::desc(), OpaqueMeshInstance::desc()],   // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: MyTexture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual, // 1.
                stencil: wgpu::StencilState::default(), // 2.
                bias: wgpu::DepthBiasState::default(),
            }), // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None,     // 6.
        });
        render_pipeline
    }

    pub fn new(device: &wgpu::Device, 
        config: &wgpu::SurfaceConfiguration,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let texture_bind_group_layout = Self::create_texture_bind_group_layout(device);
        let pipeline = Self::create_pipeline(device, config, camera_bind_group_layout, &texture_bind_group_layout);
        Self {
            pipeline,
            texture_bind_group_layout,
        }
    }

    fn create_render_pass<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
        color_view: &'a wgpu::TextureView,
        depth_view: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a>{
        let color_attachment = Some(wgpu::RenderPassColorAttachment {
            view: &color_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        });
        let depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        };
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[color_attachment],
            depth_stencil_attachment: Some(depth_stencil_attachment),
            occlusion_query_set: None,
            timestamp_writes: None,
        };
        encoder.begin_render_pass(&render_pass_descriptor)
    }

    pub fn render(&mut self, 
        renderables: &HashMap<MeshMeta, Vec<OpaqueMeshInstance>>,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_view: &wgpu::TextureView, 
        depth_view: &wgpu::TextureView,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        // begin render pass
        let mut render_pass = self.create_render_pass(encoder, color_view, depth_view);

        for (mesh_meta, instances) in renderables.iter(){
            let mesh_model = CACHE.get_with(CacheKey::OpaqueMeshMeta(mesh_meta.clone()), ||{
                let opaque_mesh_model = OpaqueMeshInfo::new(device, queue, &self.texture_bind_group_layout);
                Arc::new(CacheValue::OpaqueMeshModel(opaque_mesh_model))
            });
            let mesh_model = match mesh_model.as_ref(){
                CacheValue::OpaqueMeshModel(mesh_model) => mesh_model,
                _ => unreachable!(),
            };
            render_pass.set_pipeline(&self.pipeline);

            //needs a texture bind group from the model
            render_pass.set_bind_group(0, camera_bind_group, &[]);
            render_pass.set_bind_group(1, &mesh_model.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, mesh_model.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh_model.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            let instance_data = instances.iter().map(OpaqueMeshInstance::to_raw).collect::<Vec<_>>();
            let instance_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instance_data),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.draw_indexed(0..mesh_model.num_indices, 0, 0..instances.len() as u32);
        }
    }
}