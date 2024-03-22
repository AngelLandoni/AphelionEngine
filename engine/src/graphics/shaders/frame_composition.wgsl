struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var vertices = array<vec4<f32>, 6>(
        vec4<f32>(-1.0, 1.0, 0.0, 1.0),

        vec4<f32>(1.0, 1.0, 0.0, 1.0),

        vec4<f32>(-1.0, -1.0, 0.0, 1.0),

        vec4<f32>(1.0, 1.0, 0.0, 1.0),

        vec4<f32>(1.0, -1.0, 0.0, 1.0),

        vec4<f32>(-1.0, -1.0, 0.0, 1.0),
    );

    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),

        vec2<f32>(1.0, 0.0),

        vec2<f32>(0.0, 1.0),

        vec2<f32>(1.0, 0.0),

        vec2<f32>(1.0, 1.0),

        vec2<f32>(0.0, 1.0),
    );

    var out: VertexOutput;
    out.clip_position = vertices[in_vertex_index];
    out.tex_coords = uvs[in_vertex_index];
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}