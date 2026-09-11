struct HudUniform {
    // xy = surface size in pixels, zw unused here.
    params: vec4<f32>,
};

@group(0) @binding(0) var<uniform> hud: HudUniform;

struct Instance {
    // xy = top-left in pixels, zw = size in pixels.
    @location(0) rect: vec4<f32>,
    // fill rgba
    @location(1) fill: vec4<f32>,
    // border rgba
    @location(2) border: vec4<f32>,
    // x = border thickness (px), yzw unused
    @location(3) params: vec4<f32>,
};

struct VsOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_px: vec2<f32>,
    @location(1) size_px: vec2<f32>,
    @location(2) fill: vec4<f32>,
    @location(3) border: vec4<f32>,
    @location(4) border_thickness: f32,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32, instance: Instance) -> VsOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );

    let screen = hud.params.xy;
    let corner = corners[vertex_index];
    let origin = instance.rect.xy;
    let size = instance.rect.zw;

    let pixel = origin + corner * size;
    let ndc = vec2<f32>(
        pixel.x / screen.x * 2.0 - 1.0,
        1.0 - pixel.y / screen.y * 2.0,
    );

    var out: VsOut;
    out.clip_position = vec4<f32>(ndc, 0.0, 1.0);
    out.local_px = corner * size;
    out.size_px = size;
    out.fill = instance.fill;
    out.border = instance.border;
    out.border_thickness = instance.params.x;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let d_left = in.local_px.x;
    let d_right = in.size_px.x - in.local_px.x;
    let d_top = in.local_px.y;
    let d_bottom = in.size_px.y - in.local_px.y;
    let d_edge = min(min(d_left, d_right), min(d_top, d_bottom));

    if (d_edge <= in.border_thickness) {
        return in.border;
    }
    return in.fill;
}
