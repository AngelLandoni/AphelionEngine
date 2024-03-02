use bytemuck::{AnyBitPattern, Pod};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Adapter, Buffer, BufferAddress, BufferUsages, Device, DeviceDescriptor,
    Extent3d, Features, Limits, Queue, RequestAdapterOptions, Sampler,
    SamplerDescriptor, ShaderModule, Surface, SurfaceConfiguration,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureViewDescriptor, COPY_BUFFER_ALIGNMENT,
};

use crate::{
    graphics::{
        gpu::GpuAbstractor, BufferCreator, BufferHandler, IndexBuffer,
        ShaderHandler, SurfaceHandler, Texture, UniformBuffer, VertexBuffer,
    },
    host::window::Window,
    types::Size,
};

use super::buffer::{
    map_usages, WGPUTexture, WgpuIndexBuffer, WgpuUniformBuffer,
    WgpuVertexBuffer,
};

pub(crate) const DEPTH_TEXTURE_FORMAT: TextureFormat =
    TextureFormat::Depth32Float;

/// Holds all the essential information required for GPU interaction.
pub struct Gpu {
    pub surface: Surface,
    /// Represents a physical GPU device available in the system.
    #[allow(dead_code)]
    pub adapter: Adapter,
    /// Represents a logical device that facilitates interaction with the
    /// underlying physical GPU (Adapter).
    pub device: Device,
    pub queue: Queue,
    pub surface_config: SurfaceConfiguration,
    pub texture_format: TextureFormat,
}

impl Gpu {
    /// Creates and returns new instance of `GPU`.
    pub async fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::default();

        let surface = unsafe { instance.create_surface(window) }
            .expect("Unable to acquire the `wgpu` surface.");

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Unable to acquire the `wgpu` adapter");

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    features: Features::all_webgpu_mask(),
                    limits: Limits::default()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Unable to acquire the wgpu device and/or queue");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let texture_format = swapchain_capabilities.formats[0];

        let size = window.inner_size();

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        Gpu {
            surface,
            adapter,
            device,
            queue,
            surface_config,
            texture_format,
        }
    }

    /// Reads a shader file and generate a module.
    pub(crate) fn compile_program(
        &self,
        label: &str,
        shader_code: &str,
    ) -> ShaderModule {
        self.device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(shader_code.into()),
            })
    }

    /// Allocates and initilizes a chunk of memory on the GPU.
    pub(crate) fn allocate_buffer_init<T: Pod + AnyBitPattern>(
        &self,
        label: &str,
        content: T,
        usage: BufferUsages,
    ) -> Buffer {
        self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&[content]),
            usage,
        })
    }

    pub(crate) fn raw_allocate_buffer_init(
        &self,
        label: &str,
        content: &[u8],
        usage: BufferUsages,
    ) -> Buffer {
        self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: content,
            usage,
        })
    }

    pub(crate) fn allocate_aligned_zero_buffer(
        &self,
        label: &str,
        size: u64,
        usage: BufferUsages,
    ) -> Buffer {
        let padded_size = match size.checked_sub(COPY_BUFFER_ALIGNMENT - 1) {
            Some(subtracted) => subtracted & !(COPY_BUFFER_ALIGNMENT - 1),
            None => 0,
        };

        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: padded_size,
            usage,
            mapped_at_creation: false, // We don't need it mapped at creation
        })
    }

    pub(crate) fn write_aligned_buffer(
        &self,
        buffer: &Buffer,
        offset: BufferAddress,
        data: &[u8],
    ) {
        let aligned_offset = match offset.checked_sub(COPY_BUFFER_ALIGNMENT - 1)
        {
            Some(subtracted) => subtracted & !(COPY_BUFFER_ALIGNMENT - 1),
            None => 0,
        };
        self.queue.write_buffer(buffer, aligned_offset, data);
    }
}

impl GpuAbstractor for Gpu {}

impl ShaderHandler for Gpu {
    fn compile_program(&self) {}
}

impl SurfaceHandler for Gpu {
    fn surface_size(&self) -> crate::types::Size<u32> {
        Size::new(self.surface_config.width, self.surface_config.height)
    }
}

impl BufferCreator for Gpu {
    /// Stores the information into the GPU RAM and returns a reference to it.
    fn allocate_vertex_buffer(
        &self,
        label: &str,
        data: &[u8],
    ) -> Box<dyn VertexBuffer> {
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: data,
            usage: BufferUsages::VERTEX,
        });

        Box::new(WgpuVertexBuffer(buffer))
    }

    /// Stores the information into the GPU RAM and returns a reference to it.
    fn allocate_index_buffer(
        &self,
        label: &str,
        data: &[u8],
    ) -> Box<dyn IndexBuffer> {
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: data,
            usage: BufferUsages::INDEX,
        });

        Box::new(WgpuIndexBuffer(buffer))
    }

    fn allocate_depth_texture(
        &self,
        label: &str,
        width: u32,
        height: u32,
    ) -> Box<dyn Texture> {
        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some(label),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: DEPTH_TEXTURE_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = self.device.create_sampler(&SamplerDescriptor {
            label: Some(label),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual), // 5.
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Box::new(WGPUTexture {
            texture,
            view,
            sampler: Some(sampler),
        })
    }

    fn allocate_target_texture(
        &self,
        label: &str,
        width: u32,
        height: u32,
    ) -> Box<dyn Texture> {
        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some(label),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Box::new(WGPUTexture {
            texture,
            view,
            sampler: Some(sampler),
        })
    }

    fn allocate_cubemap_texture(
        &self,
        label: &str,
        size: u32,
    ) -> Box<dyn Texture> {
        // Allocate a new cube map of the size of the
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(format!("{} texture", label).as_ref()),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                // A cube has 6 sides, so we need 6 layers
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(format!("{} view", label).as_str()),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            ..Default::default()
        });

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(format!("{} sampler", label).as_str()),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Box::new(WGPUTexture {
            texture,
            view,
            sampler: Some(sampler),
        })
    }

    fn allocate_uniform_buffer(
        &self,
        label: &str,
        data: &[u8],
    ) -> Box<dyn UniformBuffer> {
        let buffer = self.raw_allocate_buffer_init(
            label,
            data,
            BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        );

        Box::new(WgpuUniformBuffer(buffer))
    }

    fn allocate_aligned_zero_vertex_buffer(
        &self,
        label: &str,
        size: u64,
        uses: crate::graphics::BufferUsage,
    ) -> Box<dyn VertexBuffer> {
        Box::new(WgpuVertexBuffer(self.allocate_aligned_zero_buffer(
            label,
            size,
            BufferUsages::VERTEX | map_usages(uses),
        )))
    }
}

impl BufferHandler for Gpu {
    fn write_uniform_buffer(
        &self,
        buffer: &Box<dyn UniformBuffer>,
        offset: u64,
        data: &[u8],
    ) {
        let buffer = buffer
            .downcast_ref::<WgpuUniformBuffer>()
            .expect("Unable to downcast Uniform Buffer");

        self.queue.write_buffer(&buffer.0, offset, data);
    }

    fn write_vertex_buffer(
        &self,
        buffer: &Box<dyn VertexBuffer>,
        offset: u64,
        data: &[u8],
    ) {
        let buffer = buffer
            .downcast_ref::<WgpuVertexBuffer>()
            .expect("Unable to downcast Vertex Buffer");

        self.queue.write_buffer(&buffer.0, offset, data);
    }
}
