struct TransposeUniforms {
    size: u32, // padded
}

@group(0) @binding(0) var<uniform> uniforms: TransposeUniforms;
@group(0) @binding(1) var<storage, read_write> data: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read_write> scratch: array<vec2<f32>>;

@compute @workgroup_size(16, 16, 1)
fn copy_upper(
    @builtin(global_invocation_id) g: vec3<u32>,
) {
    let fft_sz = uniforms.size;
    
    // filter out lower triangle (including diagonal) 
    if (g.x >= fft_sz || g.y >= fft_sz || g.y >= g.x) { return; }

    let src_idx = g.y * fft_sz + g.x; // upper triangle
    let dst_idx = g.x * fft_sz + g.y; // lower triangle
    let sidx = g.y * fft_sz - (g.y * (g.y + 1)) / 2 + g.x - g.y - 1;

    scratch[sidx] = data[src_idx];
    data[src_idx] = data[dst_idx]; 
}

@compute @workgroup_size(16, 16, 1)
fn transpose_lower(
    @builtin(global_invocation_id) g: vec3<u32>,
) {
    let fft_sz = uniforms.size;

    if (g.x >= fft_sz || g.y >= fft_sz || g.y >= g.x) { return; }

    let src_idx = g.y * fft_sz + g.x; // upper triangle
    let dst_idx = g.x * fft_sz + g.y; // lower triangle
    let sidx = g.y * fft_sz - (g.y * (g.y + 1)) / 2 + g.x - g.y - 1;

    data[dst_idx] = scratch[sidx];
}