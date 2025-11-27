struct FFTUniforms {
    size: u32, // power of 2, must be >= 2 * WORKGROUP_SIZE
    num_stages: u32, // log2 size
}

const PI: f32 = 3.14159265;
const MAX_SIZE: u32 = 2048u; // guaranteed minimum workgroup memory size is 4096
// const WORKGROUP_SIZE: u32 = 1u;
const WORKGROUP_SIZE: u32 = 256u;

var<workgroup> shared_data: array<vec2<f32>, MAX_SIZE>; 

@group(0) @binding(0) var<uniform> uniforms: FFTUniforms;
@group(0) @binding(1) var<storage, read_write> in_out: array<vec2<f32>>;


// Complex multiplication: (a + bi) * (c + di) = (ac - bd) + (ad + bc)i
fn complex_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

// Bit reversal for indices
fn bit_reverse(x: u32, num_bits: u32) -> u32 {
    var result = 0u;
    var value = x;
    for (var i = 0u; i < num_bits; i++) {
        result = (result << 1u) | (value & 1u);
        value = value >> 1u;
    }
    return result;
}

fn fft_1d(
    global_id: vec3<u32>,
    local_id: vec3<u32>,
    workgroup_id: vec3<u32>,
    inverse: bool,
) {
    let row = workgroup_id.x; 
    let thread_id = local_id.x;

    let num_stages = uniforms.num_stages;
    let n = uniforms.size;
    let angle_sign = select(1.0, -1.0, inverse);

    let elements_per_thread = n / WORKGROUP_SIZE; // will always divide evenly, since input is padded

    // Load row into shared memory
    for (var i = 0u; i < elements_per_thread; i++) {
        let idx = thread_id + i * WORKGROUP_SIZE;  // I am suspicious of this...
        // bit reverse index when writing to shared memory 
        let reversed_idx = bit_reverse(idx, num_stages);  
        shared_data[idx] = in_out[row * n + reversed_idx]; // TODO: scattered reads here ...
    }

    workgroupBarrier();

    for (var stage = 0u; stage < num_stages; stage++) {
        let block_size = 1u << (stage + 1u);  // 2, 4, 8, 16, 32, ...
        let half_block = block_size >> 1u;

        let total_butterflies = n / 2u;
        let butterflies_per_thread = total_butterflies / WORKGROUP_SIZE;

        for (var i = 0u; i < butterflies_per_thread; i++) {
            let butterfly_id = thread_id + i * WORKGROUP_SIZE;
            
            // Map butterfly_id to element pair
            let block_idx = butterfly_id / half_block;
            let pos_in_block = butterfly_id % half_block;

            let idx_a = block_idx * block_size + pos_in_block;
            let idx_b = idx_a + half_block;

            // Compute twiddle factor
            let angle = angle_sign * 2.0 * PI * f32(pos_in_block) / f32(block_size);
            let twiddle = vec2<f32>(cos(angle), sin(angle));

            let a = shared_data[idx_a];
            let b = shared_data[idx_b];

            let wb = complex_mul(twiddle, b);
                
            // Butterfly operation
            shared_data[idx_a] = vec2<f32>(a.x + wb.x, a.y + wb.y);
            shared_data[idx_b] = vec2<f32>(a.x - wb.x, a.y - wb.y);
        }

        workgroupBarrier();
    }

    workgroupBarrier();

    for (var i = 0u; i < elements_per_thread; i++) {
        let idx = thread_id + i * WORKGROUP_SIZE;
        if (idx < n) {
            let global_idx = row * n + idx;
            in_out[global_idx] = shared_data[idx];
        }
    }
}

@compute
// @workgroup_size(1)
@workgroup_size(256)
fn fft_forward(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    fft_1d(global_id, local_id, workgroup_id, false);
}

@compute 
// @workgroup_size(1)
@workgroup_size(256)
fn fft_inverse(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    fft_1d(global_id, local_id, workgroup_id, true);
}

