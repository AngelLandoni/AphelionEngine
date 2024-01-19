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
};

use crate::host::window::Window;

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
    pub(crate) fn create_shader(&self, label: &str, shader_code: &str) -> ShaderModule {
        self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        })
    }
}