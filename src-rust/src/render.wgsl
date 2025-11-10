struct Globals { 
    height: u32,
    width: u32,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<storage, read> colors: array<vec3<f32>>;
@group(0) @binding(2) var<storage, read> grid: array<f32>;

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

    if (x >= globals.width || y >= globals.height) {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }

    let val = grid[y * globals.width + x];
    let color_index = u32(val * 255);
    return vec4<f32>(colors[color_index], 1.0);
}