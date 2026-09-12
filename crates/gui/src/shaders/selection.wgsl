struct Camera {
    view_projection: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;

struct Instance {
    // xy = world-space center, z = radius (world px), w = unused.
    @location(0) data: vec4<f32>,
    // rgba color (premultiplied not required; blend uses standard alpha).
    @location(1) color: vec4<f32>,
};

struct VsOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32, instance: Instance) -> VsOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );

    let corner = corners[vertex_index];
    let center = instance.data.xy;
    let radius = instance.data.z;

    // Flatten more aggressively for an iso ground disc feel.
    let world = center + corner * vec2<f32>(radius, radius * 0.35);
    let clip = camera.view_projection * vec4<f32>(world, 0.0, 1.0);

    var out: VsOut;
    out.clip_position = clip;
    out.local = corner;
    out.color = instance.color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let d = length(in.local);
    if (d > 1.0) {
        discard;
    }
    // Soft edge from 0.9..1.0.
    let edge = 1.0 - smoothstep(0.9, 1.0, d);
    return vec4<f32>(in.color.rgb, in.color.a * edge);
}
