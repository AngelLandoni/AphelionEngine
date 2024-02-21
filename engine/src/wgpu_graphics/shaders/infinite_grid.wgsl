struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
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
    out.clip_position = camera.view_proj * vertices[in_vertex_index];
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}