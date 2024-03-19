struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    /*let c = i32(round(in.position.x * 5.0) + round(in.position.y * 5.0));
    let a = c % 2;
    let final_color = f32(a) / 2.0 + 0.3;
    return vec4<f32>(final_color, final_color, final_color, 1.0);*/

    // Define checkerboard properties
    let tileSize: f32 = 50.0; // Adjust the size of each checker tile
    let col = floor(in.position.x / tileSize);
    let row = floor(in.position.y / tileSize);
    let checker = (i32(col) + i32(row)) % 2; // Checker pattern (0 or 1)

    // Apply checkerboard pattern
    if (checker == 0) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0); // White color for one set of tiles
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black color for the other set of tiles
    }
}
