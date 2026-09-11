struct HudUniform {
    // xy = surface size in pixels, z = glyphs in the atlas, w = glyph pixel size.
    params: vec4<f32>,
};

@group(0) @binding(0) var atlas_texture: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;
@group(0) @binding(2) var<uniform> hud: HudUniform;

struct Instance {
    // xy = top-left position in pixels, z = glyph index, w = unused.
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

    let screen = hud.params.xy;
    let atlas_count = hud.params.z;
    let glyph_size = hud.params.w;

    let corner = corners[vertex_index];
    let origin = instance.data.xy;
    let glyph_index = instance.data.z;

    let pixel = origin + corner * glyph_size;
    let ndc = vec2<f32>(
        pixel.x / screen.x * 2.0 - 1.0,
        1.0 - pixel.y / screen.y * 2.0,
    );
    let cell_width = 1.0 / atlas_count;
    let u = (glyph_index + corner.x) * cell_width;
    let v = corner.y;

    var out: VsOut;
    out.clip_position = vec4<f32>(ndc, 0.0, 1.0);
    out.uv = vec2<f32>(u, v);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Font atlas stores coverage in the red channel; render as white text.
    let coverage = textureSample(atlas_texture, atlas_sampler, in.uv).r;
    return vec4<f32>(1.0, 1.0, 1.0, coverage);
}
