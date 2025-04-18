// it contains a RenderPipeline, can store drawables, and render them
// different pipelines have different binding requirements, so the model types are different

use std::{collections::BTreeSet, sync::Arc};

use wgpu::{RenderPipeline, util::DeviceExt};

use crate::{
    my_texture::MyTexture,
    ui_renderable::{SortOrder, UIInstance, UIInstanceRaw, UIRenderable},
};

// model
// mesh_num
pub struct UIPipeline {
    pub pipeline: RenderPipeline,
    pub material_bind_group_layout: wgpu::BindGroupLayout,
}

impl UIPipeline {
    fn create_material_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Material Bind Group Layout"),
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
    pub fn create_material_bind_group(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &MyTexture,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.material_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("ui_material_bind_group"),
        })
    }
    fn create_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        material_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[material_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ui.wgsl").into()),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),      // 1.
                buffers: &[UIInstanceRaw::desc()], // 2.
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
                stencil: wgpu::StencilState::default(),          // 2.
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

    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let material_bind_group_layout = Self::create_material_bind_group_layout(device);
        let pipeline = Self::create_pipeline(device, config, &material_bind_group_layout);
        Self {
            pipeline,
            material_bind_group_layout,
        }
    }

    fn create_render_pass<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
        color_view: &'a wgpu::TextureView,
        depth_view: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a> {
        let color_attachment = Some(wgpu::RenderPassColorAttachment {
            view: &color_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
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

    pub fn create_index_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        let indices: [u16; 6] = [0, 1, 2, 3, 4, 5];
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        })
    }

    pub fn render(
        &mut self,
        renderables: &Vec<(&UIRenderable, Arc<Vec<UIInstance>>)>,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView, // use depth to sort
        index_buffer: &wgpu::Buffer,
    ) {
        let mut sort_orders = BTreeSet::<SortOrder>::new();
        for (_, instances) in renderables.iter() {
            for instance in instances.iter() {
                sort_orders.insert(instance.sort_order);
            }
        }
        let sort_order_to_depth = sort_orders
            .iter()
            .enumerate()
            .map(|(i, sort_order)| (sort_order.clone(), 1.0 - i as f32 / sort_orders.len() as f32))
            .collect::<std::collections::HashMap<_, _>>();
        // begin render pass
        let mut render_pass = self.create_render_pass(encoder, color_view, depth_view);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        //needs a texture bind group from the model
        for (ui_renderable, instances) in renderables.iter() {
            render_pass.set_bind_group(0, &ui_renderable.material_bind_group, &[]);
            let instance_data = instances
                .iter()
                .map(|instance| instance.to_raw(&sort_order_to_depth))
                .collect::<Vec<_>>();
            let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            });
            render_pass.set_vertex_buffer(0, instance_buffer.slice(..));
            render_pass.draw_indexed(0..6, 0, 0..instances.len() as u32);
        }
    }
}
