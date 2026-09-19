use core::time::Duration;

pub const ORDER_MARKER_RENDER_DURATION: Duration = Duration::from_millis(600);
pub const DEBUG_PRAGMA: &str = "\
    wgpu=info,\
    naga=info,\
    winit=info,\
    winit::platform_impl::linux::x11::xdisplay=error,\
    wgpu_hal::vulkan::instance=error,\
    wgpu_hal::vulkan::adapter=warn,\
    winit::platform_impl::linux::x11::window=warn,\
    debug";
