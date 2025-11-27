struct RandomnessUniforms {
    height: u32,
    width: u32,
    x: u32,
    y: u32,
    seed: u32,
    density: f32,
    use_brush: u32,
    size: u32,
}

@group(0) @binding(0) var<uniform> uniforms: RandomnessUniforms;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

// https://www.reedbeta.com/blog/quick-and-easy-gpu-random-numbers-in-d3d11/
fn wang_hash(seed: u32) -> u32 {
    var s = seed;
    s = (s ^ 61u) ^ (s >> 16u);
    s = s * 9u;
    s = s ^ (s >> 4u);
    s = s * 0x27d4eb2du;
    s = s ^ (s >> 15u);
    return s;
}

@compute
@workgroup_size(16, 16)
fn randomize(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= uniforms.width || y >= uniforms.height) {
        return;
    }

    if (uniforms.use_brush == 1 && !(
        i32(uniforms.x) - i32(uniforms.size) <= i32(x) &&  uniforms.x + uniforms.size >= x &&
        i32(uniforms.y) - i32(uniforms.size) <= i32(y) &&  uniforms.y + uniforms.size >= y
    )) {
        return;
    }

    let index = y * uniforms.width + x;
    let combined = x + y * uniforms.width + uniforms.seed * uniforms.width * uniforms.height;
    output[index] = (f32(wang_hash(combined)) / 4294967296.0) * uniforms.density;
}