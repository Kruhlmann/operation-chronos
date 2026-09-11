struct SpriteUniform {
    // x = columns, y = rows, z = sprite index, w = surface aspect (w/h).
    params: vec4<f32>,
};

@group(0) @binding(0) var sprite_texture: texture_2d<f32>;
@group(0) @binding(1) var sprite_sampler: sampler;
@group(0) @binding(2) var<uniform> sprite: SpriteUniform;

struct VsOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, -0.5),
        vec2<f32>( 0.5, -0.5),
        vec2<f32>( 0.5,  0.5),
        vec2<f32>(-0.5, -0.5),
        vec2<f32>( 0.5,  0.5),
        vec2<f32>(-0.5,  0.5),
    );

    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 0.0),
    );

    let columns = sprite.params.x;
    let rows = sprite.params.y;
    let index = sprite.params.z;
    let aspect = sprite.params.w;
    let col = floor(index % columns);
    let row = floor(index / columns);

    let cell_size = vec2<f32>(1.0 / columns, 1.0 / rows);
    let cell_origin = vec2<f32>(col, row) * cell_size;
    let uv = cell_origin + uvs[idx] * cell_size;

    var pos = positions[idx];
    if (aspect >= 1.0) {
        pos.x = pos.x / aspect;
    } else {
        pos.y = pos.y * aspect;
    }

    var out: VsOut;
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(sprite_texture, sprite_sampler, in.uv);
}
