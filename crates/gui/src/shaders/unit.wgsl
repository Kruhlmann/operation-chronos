struct Camera {
    view_projection: mat4x4<f32>,
};

struct Sheet {
    // x = sheet width px, y = sheet height px, z = frame width px, w = frame height px.
    params: vec4<f32>,
};

@group(0) @binding(0) var sheet_texture: texture_2d<f32>;
@group(0) @binding(1) var sheet_sampler: sampler;
@group(0) @binding(2) var<uniform> camera: Camera;
@group(0) @binding(3) var<uniform> sheet: Sheet;

struct Instance {
    // xy = world-pixel top-left anchor, z = frame index, w = unused.
    @location(0) data: vec4<f32>,
};

struct VsOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
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

    let corner = corners[vertex_index];
    let anchor = instance.data.xy;
    let frame = instance.data.z;

    let sheet_w = sheet.params.x;
    let sheet_h = sheet.params.y;
    let frame_w = sheet.params.z;
    let frame_h = sheet.params.w;

    let world = anchor + corner * vec2<f32>(frame_w, frame_h);
    let clip = camera.view_projection * vec4<f32>(world, 0.0, 1.0);

    let u0 = frame * frame_w / sheet_w;
    let u1 = (frame + 1.0) * frame_w / sheet_w;
    let v0 = 0.0;
    let v1 = frame_h / sheet_h;

    var out: VsOut;
    out.clip_position = clip;
    out.uv = vec2<f32>(mix(u0, u1, corner.x), mix(v0, v1, corner.y));
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(sheet_texture, sheet_sampler, in.uv);
}
