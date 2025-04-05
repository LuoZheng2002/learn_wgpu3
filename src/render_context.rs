use std::{collections::HashMap, sync::Arc};

use tokio::runtime::Runtime;
use wgpu::{util::DeviceExt, CompositeAlphaMode};
use winit::window::Window;

use crate::{
    cache::{CacheKey, CacheValue, CACHE}, camera_uniform::CameraUniform, light_uniform::LightUniform, model_data::{self, MeshMeta, ModelData, MyMesh}, model_instance::ModelInstance, my_texture::MyTexture, opaque_pipeline::{self, OpaquePipeline}, state::State
};

pub struct RenderContext {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    pub camera_buffer: wgpu::Buffer,
    // most pipelines will use this
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
    // light stuff
    pub light_buffer: wgpu::Buffer,
    pub depth_texture: MyTexture,

    pub opaque_pipeline: OpaquePipeline,
}

impl RenderContext {
    pub fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // instance represents the connection to the graphics API and system GPU drivers
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        // surface only depends on window
        let surface = instance.create_surface(window).unwrap();
        // tokio runtime for blocking async tasks
        let runtime = Runtime::new().unwrap();
        // adapter represents a GPU
        let adapter = runtime
            .block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }))
            .unwrap();

        let (device, queue) = runtime
            .block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None, // Trace path
            ))
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        // define how the surface creates its underlying SurfaceTextures
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            // enable vsync with fifo present mode
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let camera_uniform = CameraUniform::default();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let depth_texture = MyTexture::create_depth_texture(&device, &config, "depth texture");

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("view_bind_group_layout"),
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        let light_uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };
        let light_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[light_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let opaque_pipeline = OpaquePipeline::new(&device, &config, &camera_bind_group_layout, &light_buffer);
        RenderContext {
            surface,
            device,
            queue,
            config,
            size,
            opaque_pipeline,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            depth_texture,
            light_buffer,
        }
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth_texture =
            MyTexture::create_depth_texture(&self.device, &self.config, "depth texture");
    }

    pub fn render(&mut self, state: &mut State) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // update camera transform
        let aspect = self.config.width as f32 / self.config.height as f32;
        let camera_uniform = CameraUniform::new(&state.camera, aspect, true);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
        // update light transform
        let light_uniform = LightUniform {
            position: state.light_position.into(),
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };
        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[light_uniform]),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        // convert model instances to mesh instances
        let mut opaque_meshes = Vec::<(Arc<MyMesh>, Vec<Arc<ModelInstance>>)>::new();
        for (model_meta, instances) in state.render_submissions.iter() {
            // need to get the model info to determine which meshes are opaque
            let model_data = CACHE.get_with(CacheKey::ModelMeta(model_meta.clone()), || {
                let model_data = model_meta.load_model(
                    &self.device,
                    &self.queue,
                    &self.opaque_pipeline,
                );
                Arc::new(CacheValue::ModelData(model_data))
            });
            let model_data = match model_data.as_ref() {
                CacheValue::ModelData(model_info) => model_info,
                _ => unreachable!(),
            };
            for opaque_mesh in model_data.opaque_meshes.iter() {
                
                opaque_meshes.push((opaque_mesh.clone(), instances.clone()));
            }
        }

        self.opaque_pipeline.render(
            &opaque_meshes,
            &mut encoder,
            &self.device,
            &self.queue,
            &view,
            &self.depth_texture.view,
            &self.camera_bind_group,
        );

        state.render_submissions.clear();

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
