lazy_static::lazy_static! {
    pub static ref LOG_LEVEL: &'static str = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    pub static ref LOG_FILTER: String = format!(
        "wgpu=info,\
        naga=info,\
        winit=info,\
        winit::platform_impl::linux::x11::xdisplay=error,\
        wgpu_hal::vulkan::instance=error,\
        wgpu_hal::vulkan::adapter=warn,\
        winit::platform_impl::linux::x11::window=warn,\
        {}", *LOG_LEVEL);
}
