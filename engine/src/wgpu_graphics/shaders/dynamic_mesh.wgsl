struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
};

struct TransformUniform {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct TransformInput {
    @location(2) t0: vec4<f32>,
    @location(3) t1: vec4<f32>,
    @location(4) t2: vec4<f32>,
    @location(5) t3: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    transform: TransformInput,
) -> VertexOutput {
    let transform_matrix = mat4x4<f32>(
        transform.t0,
        transform.t1,
        transform.t2,
        transform.t3,
    );

    var out: VertexOutput;
    out.color = model.color;
    out.position = camera.view_proj * transform_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
