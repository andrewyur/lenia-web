struct Parameters { 
    height: u32,
    width: u32
}

@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var<storage, read> grid: array<f32>;

@vertex fn vs(
    @builtin(vertex_index) vertexIndex : u32
) -> @builtin(position) vec4f {
    let pos = array(
        vec2f(-1.0, -1.0),  // bottom left
        vec2f( 1.0, -1.0),  // bottom right
        vec2f(-1.0,  1.0),  // top left
        vec2f( 1.0,  1.0),  // top right
    );

    return vec4f(pos[vertexIndex], 0.0, 1.0);
}

@fragment fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4f {
    let x = u32(pos.x);
    let y = u32(pos.y);

    if (x >= parameters.width || y >= parameters.height) {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }

    let val = grid[y * parameters.width + x];
    return vec4<f32>(val * 0.7 - 0.2, val / 0.3 + 0., val * 0.5 + 0.2, 1.0);
}