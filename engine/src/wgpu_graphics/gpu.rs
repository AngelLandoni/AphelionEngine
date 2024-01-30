use bytemuck::{Pod, AnyBitPattern};
use shipyard::Unique;
use wgpu::{
    Surface,
    Adapter,
    Device,
    Queue,
    RequestAdapterOptions,
    Features,
    Limits,
    DeviceDescriptor,
    SurfaceConfiguration,
    TextureUsages,
    ShaderModule, 
    TextureFormat,
    Buffer,
    BufferAddress,
    BufferUsages,
    BufferDescriptor,
    util::{DeviceExt, BufferInitDescriptor},
    COPY_BUFFER_ALIGNMENT,
};

use crate::{
    graphics::{
        gpu::GpuAbstractor, BufferCreator, IndexBuffer, ShaderHandler, VertexBuffer
    }, 
    host::window::Window
};

use super::buffer::{WgpuIndexBuffer, WgpuVertexBuffer};

/// Holds all the essential information required for GPU interaction.
pub(crate) struct Gpu {
    pub surface: Surface,
    /// Represents a physical GPU device available in the system.
    pub adapter: Adapter,
    /// Represents a logical device that facilitates interaction with the
    /// underlying physical GPU (Adapter).
    pub device: Device,
    pub queue: Queue,
    pub surface_config: SurfaceConfiguration,
    pub texture_format: TextureFormat
}

impl Gpu {
    /// Creates and returns new instance of `GPU`.
    pub async fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::default();

        let surface = unsafe {
            instance.create_surface(window)
        }
        .expect("Unable to acquire the `wgpu` surface.");

        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Unable to acquire the `wgpu` adapter");

        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            label: None,
            features: Features::empty(),
            limits: Limits::default()
                .using_resolution(adapter.limits())
            }, None
        )
        .await
        .expect("Unable to acquire the wgpu device and/or queue");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let texture_format = swapchain_capabilities.formats[0];

        let size = window.accesor.inner_size();

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
    pub(crate) fn compile_program(&self, label: &str, shader_code: &str) -> ShaderModule {
        self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        })
    }

    /// Allocates a buffer in the GPU RAM and returns a reference to it.
    pub(crate) fn allocate_buffer(&self, label: &str, size: u64, usage: BufferUsages) -> Buffer {
        // Convert the size from the provided one into one that WGPU handles.
        let unpadded_size: BufferAddress = size as BufferAddress;
        // Make sure the size is 4 bytes aligned.
        let padding: BufferAddress = 
            COPY_BUFFER_ALIGNMENT -
            unpadded_size %
            COPY_BUFFER_ALIGNMENT;

        // Final padding, the size now is memory aligned.
        let padded_size: BufferAddress = unpadded_size + padding;

        // Allocate and return the reference.
        self.device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size: padded_size,
            usage,
            mapped_at_creation: true,
        })
    }

    /// Allocates and initilizes a chunk of memory on the GPU.
    pub(crate) fn allocate_buffer_init<T: Pod + AnyBitPattern>(
        &self,
        label: &str,
        content: T,
        usage: BufferUsages
    ) -> Buffer {
        self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&[content]),
            usage,
        })
    }
}

impl GpuAbstractor for Gpu {}

impl ShaderHandler for Gpu {
    fn compile_program(&self) {
        
    }
}

impl BufferCreator for Gpu {
    /// Stores the information into the GPU RAM and returns a reference to it.
    fn create_vertex_buffer(
        &self,
        label: &str,
        data: &[u8]
    ) -> Box<dyn VertexBuffer> {
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: data,
            usage: BufferUsages::VERTEX,
        });

        Box::new(WgpuVertexBuffer(buffer))
    }

    /// Stores the information into the GPU RAM and returns a reference to it.
    fn create_index_buffer(
        &self, label: &str,
        data: &[u8]
    ) -> Box<dyn IndexBuffer> {
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: data,
            usage: BufferUsages::INDEX,
        });

        Box::new(WgpuIndexBuffer(buffer))
    }
}