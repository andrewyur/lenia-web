struct KernelUniforms {
    size: u32,
}

@group(0) @binding(0) var<uniform> uniforms: KernelUniforms;
@group(0) @binding(1) var<storage, read_write> in_out: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read> kernel: array<vec2<f32>>;

fn complex_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

@compute
@workgroup_size(16, 16, 1)
fn apply_kernel(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= uniforms.size || y >= uniforms.size) {
        return;
    }
    
    let idx = y * uniforms.size + x;

    in_out[idx] = complex_mul(in_out[idx], kernel[idx]);
}