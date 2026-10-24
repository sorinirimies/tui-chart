// Synthwave horizon grid: neon lines receding to a glowing horizon.
@group(0) @binding(0) var<uniform> time: vec4<f32>;
@group(0) @binding(1) var<uniform> rect: vec4<f32>;

@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let horizon = 0.38;
    var col = vec3<f32>(0.02, 0.0, 0.06);
    if (uv.y > horizon) {
        let depth = (uv.y - horizon) / (1.0 - horizon);
        let z = 1.0 / max(depth, 0.02);
        let x = (uv.x - 0.5) * z * 4.0;
        let gz = fract(z * 0.6 - time.x * 0.5);
        let gx = fract(x);
        let line = max(1.0 - smoothstep(0.0, 0.06, abs(gz - 0.5) - 0.44),
                       1.0 - smoothstep(0.0, 0.06, abs(gx - 0.5) - 0.46));
        let fade = clamp(depth * 1.6, 0.0, 1.0);
        col = col + vec3<f32>(0.9, 0.1, 0.8) * line * fade;
    }
    let glow = exp(-abs(uv.y - horizon) * 14.0);
    col = col + vec3<f32>(0.1, 0.7, 0.9) * glow * 0.6;
    return vec4<f32>(col, 1.0);
}
