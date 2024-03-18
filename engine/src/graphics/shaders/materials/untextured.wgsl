struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // TODO(Angel): Do quads.
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
