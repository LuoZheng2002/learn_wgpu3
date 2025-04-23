// it contains a RenderPipeline, can store drawables, and render them
// different pipelines have different binding requirements, so the model types are different

use std::{path::Iter, sync::Arc};

use wgpu::{RenderPipeline, util::DeviceExt};

use crate::{
    cache::{CacheKey, CacheValue, CACHE},
    my_texture::{MyTexture, TextureSource},
    ui_node::UIRenderInstruction,
    ui_renderable::{create_placeholder_texture, UIInstance, UIInstanceRaw},
};

// model
// mesh_num
pub struct UIPipeline {
    pub pipeline: RenderPipeline,
    pub material_bind_group_layout: wgpu::BindGroupLayout,
    pub index_buffer: wgpu::Buffer,
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
        // queue: &wgpu::Queue,
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
            depth_stencil: None,
            // Some(wgpu::DepthStencilState {
            //     format: MyTexture::DEPTH_FORMAT,
            //     depth_write_enabled: true,
            //     depth_compare: wgpu::CompareFunction::LessEqual, // 1.
            //     stencil: wgpu::StencilState::default(),          // 2.
            //     bias: wgpu::DepthBiasState::default(),
            // }), // 1.
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
        let index_buffer = Self::create_index_buffer(device);
        Self {
            pipeline,
            material_bind_group_layout,
            index_buffer,
        }
    }

    fn create_render_pass<'a>(
        encoder: &'a mut wgpu::CommandEncoder,
        color_view: &'a wgpu::TextureView,
        // depth_view: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a> {
        let color_attachment = Some(wgpu::RenderPassColorAttachment {
            view: &color_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        });
        // let depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
        //     view: depth_view,
        //     depth_ops: Some(wgpu::Operations {
        //         load: wgpu::LoadOp::Clear(1.0),
        //         store: wgpu::StoreOp::Store,
        //     }),
        //     stencil_ops: None,
        // };
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[color_attachment],
            depth_stencil_attachment: None, // Some(depth_stencil_attachment),
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
    /// render_helper renders the texture specified by render_instruction to the outer texture
    pub fn render_helper<'a>(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        render_instruction: UIRenderInstruction,
        parent_texture_view: impl Into<&'a wgpu::TextureView>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_to_screen: bool,
    ) {
        let version = render_instruction.version;
        let id = render_instruction.id;
        // if there is no child texture in the cache,
        // create the blank texture and execute the sub instructions
        // tmp
        let child_texture = CACHE.get(&CacheKey::Texture(TextureSource::UI { version, id: id.clone() }));
        // let child_texture = None;
        let child_texture = match child_texture {
            Some(child_texture) => {
                child_texture
            }
            None => {
                let texture_width = u32::max(render_instruction.texture_width, 1);
                let texture_height = u32::max(render_instruction.texture_height, 1);
                let texture = MyTexture::create_render_attachment_texture(
                    device,
                    texture_width,
                    texture_height,
                    Some("ui_texture"),
                );
                
                let ui_renderable = render_instruction
                    .texture_meta
                    .to_ui_renderable(device, queue, self);
                let material_bind_group = ui_renderable.material_bind_group;
                // queue the rendering of the child texture
                let mut render_pass = Self::create_render_pass(encoder, &texture.view);
                render_pass.set_pipeline(&self.pipeline);
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);                
                render_pass.set_bind_group(0, &material_bind_group, &[]);
                let ui_instance = UIInstance {
                    location_left: -1.0,
                    location_right: 1.0,
                    location_top: 1.0,
                    location_bottom: -1.0,
                    flip_vertically: true, // render to texture
                };
                let instance_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Instance Buffer"),
                        contents: bytemuck::cast_slice(&[ui_instance.to_raw()]),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                render_pass.set_vertex_buffer(0, instance_buffer.slice(..));
                render_pass.draw_indexed(0..6, 0, 0..1);
                drop(render_pass);
                // device.poll(wgpu::Maintain::Wait);
                // call the sub instructions before rendering the child texture so that they are queued first
                for sub_instruction in render_instruction.sub_instructions {
                    self.render_helper(encoder, sub_instruction, &texture.view, device, queue, false);
                }
                let result = Arc::new(CacheValue::Texture(texture));
                CACHE.insert(
                    CacheKey::Texture(TextureSource::UI { version, id }),
                    result.clone(),
                );
                
                result
            }            
        };
        let child_texture = match child_texture.as_ref() {
            CacheValue::Texture(texture) => texture,
            _ => unreachable!(),
        };
        let normalized_location_left = render_instruction.location_left * 2.0 - 1.0;
        let normalized_location_right = render_instruction.location_right * 2.0 - 1.0;
        let normalized_location_top = -(render_instruction.location_top * 2.0 - 1.0);
        let normalized_location_bottom = -(render_instruction.location_bottom * 2.0 - 1.0);

        // let normalized_location_left = -0.5;
        // let normalized_location_right = 0.5;
        // let normalized_location_top = 0.5;
        // let normalized_location_bottom = -0.5;
        
        let ui_instance = UIInstance {
            location_left: normalized_location_left,
            location_top: normalized_location_top,
            location_right: normalized_location_right,
            location_bottom: normalized_location_bottom,
            flip_vertically: !render_to_screen,
        };
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&[ui_instance.to_raw()]),
            usage: wgpu::BufferUsages::VERTEX,
        });
        // tmp
        // let placeholder_texture = CACHE.get_with(
        //     CacheKey::PlaceholderTexture,
        //     || {
        //         let texture = MyTexture::load(TextureSource::FilePath("assets/piggies.webp".into()), device, queue).unwrap();
        //         Arc::new(CacheValue::Texture(texture))
        //     },
        // );
        // let placeholder_texture = match placeholder_texture.as_ref() {
        //     CacheValue::Texture(texture) => texture,
        //     _ => unreachable!(),
        // };



        let mut render_pass = Self::create_render_pass(encoder, parent_texture_view.into());
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_bind_group(
            0,
            &self.create_material_bind_group(device, child_texture),
            &[],
        );
        render_pass.set_vertex_buffer(0, instance_buffer.slice(..));
        render_pass.draw_indexed(0..6, 0, 0..1);
        drop(render_pass);


        // device.poll(wgpu::Maintain::Wait);
    }
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        render_instructions: Vec<UIRenderInstruction>,       
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_view: &wgpu::TextureView,
        // depth_view: &wgpu::TextureView, // use depth to sort
    ) {
        for render_instruction in render_instructions {
            self.render_helper(encoder, render_instruction, color_view, device, queue, true);
        }

        // begin render pass
        // let mut render_pass = self.create_render_pass(encoder, color_view, depth_view);
        // render_pass.set_pipeline(&self.pipeline);
        // render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        // //needs a texture bind group from the model
        // for (ui_renderable, instances) in renderables.iter() {
        //     render_pass.set_bind_group(0, &ui_renderable.material_bind_group, &[]);
        //     let instance_data = instances
        //         .iter()
        //         .map(|instance| instance.to_raw())
        //         .collect::<Vec<_>>();
        //     let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //         label: Some("Instance Buffer"),
        //         contents: bytemuck::cast_slice(&instance_data),
        //         usage: wgpu::BufferUsages::VERTEX,
        //     });
        //     render_pass.set_vertex_buffer(0, instance_buffer.slice(..));
        //     render_pass.draw_indexed(0..6, 0, 0..instances.len() as u32);
        // }
    }
}
