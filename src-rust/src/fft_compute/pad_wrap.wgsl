struct PadWrapUniforms {
    width: u32,
    height: u32,
    size: u32
}

@group(0) @binding(0) var<uniform> uniforms: PadWrapUniforms;
@group(0) @binding(1) var<storage, read> input: array<f32>;
@group(0) @binding(2) var<storage, read_write> output: array<vec2<f32>>;

@compute @workgroup_size(16, 16) // each thread owns 1 grid entry
fn pad_and_wrap(
    @builtin(global_invocation_id) g: vec3<u32>,
) {
    let size = uniforms.size;
    let width = uniforms.width;
    let height = uniforms.height;

    // centered offset
    let off_x = (size - width) / 2u;
    let off_y = (size - height) / 2u;

    let x = g.x;
    let y = g.y;

    // for (var dy = 0u; dy < size; dy += height) {
    //     for (var dx = 0u; dx < size; dx += width) {
    //         let dst_x = (x + dx) % size;
    //         let dst_y = (y + dy) % size;
    //         output[dst_y * size + dst_x] = vec2<f32>(0.0, 0.0);
    //     }
    // }

    if (x >= width || y >= height) { 
        output[y * size + x] = vec2<f32>(0.0, 0.0);
        return;
    }

    output[y * size + x] = vec2<f32>(input[y * width + x], 0.0);
}