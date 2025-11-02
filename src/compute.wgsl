struct Parameters { 
    height: u32,
    width: u32,
    t: u32,
    r: u32
}

@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var<storage, read> input: array<f32>;
@group(0) @binding(2) var<storage, read_write> output: array<f32>;

var<workgroup> tile: array<f32, 676>; // radius 5: (5 + 16 + 5)(5 + 16 + 5)

const RADIUS = 5u;
const TILESIZE = 26u;
const MULTIPLIER = 1.0 / (11.0 * 11.0);

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

    // Load with stride
    for (var stride_y = 0; stride_y < 2; stride_y++) {
        for (var stride_x = 0; stride_x < 2; stride_x++) {
            let tile_x = u32(lx) + u32(stride_x) * 16u;
            let tile_y = u32(ly) + u32(stride_y) * 16u;
            
            if (tile_x < TILESIZE && tile_y < TILESIZE) {
                let global_x = workgroup_origin_x + i32(tile_x) - i32(RADIUS);
                let global_y = workgroup_origin_y + i32(tile_y) - i32(RADIUS);

                let wrapped_x = (global_x + i32(parameters.width)) % i32(parameters.width);
                let wrapped_y = (global_y + i32(parameters.height)) % i32(parameters.height);
                
                tile[tile_y * TILESIZE + tile_x] = 
                    input[u32(wrapped_y) * parameters.width + u32(wrapped_x)];
            }
        }
    }
    
    workgroupBarrier();
    
    if(gx >= i32(parameters.width) || gy >= i32(parameters.height)) {
        return;
    }

    var sum = 0.0;
    for (var dy = 0u; dy < 11u; dy++) {
        for (var dx = 0u; dx < 11u; dx++) {
            if (dx == 5 && dy == 5) {
                continue;
            }
            let tile_idx = (u32(ly) + dy) * TILESIZE + (u32(lx) + dx);
            sum += tile[tile_idx] * MULTIPLIER;
        }
    }

    let orig = tile[(u32(ly) + RADIUS) * TILESIZE + (u32(lx) + RADIUS)];
    let growth = f32( sum>=0.12 && sum<=0.15 ) - f32( sum<0.12 || sum>0.15 );
    output[u32(gy) * parameters.width + u32(gx)] = clamp(orig + (1.0/f32(parameters.t)) * growth, 0.0, 1.0);
}