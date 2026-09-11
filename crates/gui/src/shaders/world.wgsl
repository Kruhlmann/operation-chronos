// World tiles: draws one instanced quad per drawable tile.
// Each instance carries its tile position and sprite index; a camera
// view-projection matrix maps tile space to clip space.

struct Camera {
    view_projection: mat4x4<f32>,
};

struct Sheet {
    // x = columns, y = rows in the sprite sheet.
    params: vec4<f32>,
};

@group(0) @binding(0) var sheet_texture: texture_2d<f32>;
@group(0) @binding(1) var sheet_sampler: sampler;
@group(0) @binding(2) var<uniform> camera: Camera;
@group(0) @binding(3) var<uniform> sheet: Sheet;

struct Instance {
    // xy = tile position (tile coords), z = sprite index, w = unused.
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
    let tile_pos = instance.data.xy;
    let sprite_index = instance.data.z;

    // World/tile-space position of this vertex (one tile unit per tile).
    let world = tile_pos + corner;
    let clip = camera.view_projection * vec4<f32>(world, 0.0, 1.0);

    // Select the sprite cell within the sheet.
    let columns = sheet.params.x;
    let rows = sheet.params.y;
    let col = floor(sprite_index % columns);
    let row = floor(sprite_index / columns);
    let cell = vec2<f32>(1.0 / columns, 1.0 / rows);
    let uv = (vec2<f32>(col, row) + corner) * cell;

    var out: VsOut;
    out.clip_position = clip;
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(sheet_texture, sheet_sampler, in.uv);
}
