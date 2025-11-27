struct GrowthUniforms {
    time_step: u32,
    m: f32,
    s: f32,
    fft_size: u32,
    height: u32,
    width: u32,
}

@group(0) @binding(0) var<uniform> uniforms: GrowthUniforms;
@group(0) @binding(1) var<storage, read> neighbors_sum: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read_write> in_out: array<f32>;

@compute
@workgroup_size(16, 16)
fn growth(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let fft_size = uniforms.fft_size;
    let width = uniforms.width;
    let height = uniforms.height;

    let x = global_id.x;
    let y = global_id.y;

    if(x >= width || y >= height) {
        return;
    }

    let sum = neighbors_sum[y * fft_size + x].x / f32(fft_size * fft_size);
    // in_out[y * width + x] = sum;
    let z = (sum - uniforms.m) / uniforms.s;
    let growth = exp(-0.5 * z * z) * 2.0 - 1.0;

    in_out[y * width + x] = clamp(in_out[y * width + x] + (1.0/f32(uniforms.time_step)) * growth, 0.0, 1.0);
}
