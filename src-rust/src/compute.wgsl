struct Globals { 
    height: u32,
    width: u32,
}

struct ComputeUniforms {
    time_step: u32,
    kernel_size: u32,
    kernel_sum: f32,
    m: f32,
    s: f32,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<uniform> uniforms: ComputeUniforms;
@group(0) @binding(2) var<storage, read> input: array<f32>;
@group(0) @binding(3) var<storage, read_write> output: array<f32>;
@group(0) @binding(4) var<storage> kernel: array<f32>;

var<workgroup> tile: array<f32, 6400>; // max radius 32 with workgroup size 16

@compute
@workgroup_size(16, 16)
fn cs(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let gx = i32(global_id.x);
    let gy = i32(global_id.y);
    let lx = i32(local_id.x);
    let ly = i32(local_id.y);
    
    let workgroup_origin_x = gx - lx;
    let workgroup_origin_y = gy - ly;

    let radius = u32(uniforms.kernel_size / 2);
    let tile_size = 16 + 2 * radius;


    // Load with stride
    for (var stride_y = 0; stride_y < 2; stride_y++) {
        for (var stride_x = 0; stride_x < 2; stride_x++) {
            let tile_x = u32(lx) + u32(stride_x) * 16u;
            let tile_y = u32(ly) + u32(stride_y) * 16u;
            
            if (tile_x < tile_size && tile_y < tile_size) {
                let global_x = workgroup_origin_x + i32(tile_x) - i32(radius);
                let global_y = workgroup_origin_y + i32(tile_y) - i32(radius);

                let wrapped_x = (global_x + i32(globals.width)) % i32(globals.width);
                let wrapped_y = (global_y + i32(globals.height)) % i32(globals.height);
                
                tile[tile_y * tile_size + tile_x] = 
                    input[u32(wrapped_y) * globals.width + u32(wrapped_x)];
            }
        }
    }
    
    workgroupBarrier();
    
    if(gx >= i32(globals.width) || gy >= i32(globals.height)) {
        return;
    }

    var sum = 0.0;
    for (var dy = 0u; dy < uniforms.kernel_size; dy++) {
        for (var dx = 0u; dx < uniforms.kernel_size; dx++) {
            if (dx == radius && dy == radius) {
                continue;
            }
            let tile_idx = (u32(ly) + dy) * tile_size + (u32(lx) + dx);
            sum += tile[tile_idx] * kernel[u32(dy) * uniforms.kernel_size + u32(dx)];
        }
    }

    sum /= uniforms.kernel_sum;

    let orig = tile[(u32(ly) + radius) * tile_size + (u32(lx) + radius)];

    let z = (sum - uniforms.m) / uniforms.s;
    let growth = exp(-0.5 * z * z) * 2.0 - 1.0;

    output[u32(gy) * globals.width + u32(gx)] = clamp(orig + (1.0/f32(uniforms.time_step)) * growth, 0.0, 1.0);
}