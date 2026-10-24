// Classic plasma in cyberpunk magenta/cyan.
@group(0) @binding(0) var<uniform> time: vec4<f32>;
@group(0) @binding(1) var<uniform> rect: vec4<f32>;

@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let t = time.x * 0.6;
    let v = sin(uv.x * 8.0 + t) + sin(uv.y * 6.0 - t * 1.3)
          + sin((uv.x + uv.y) * 5.0 + t * 0.7)
          + sin(length(uv - vec2<f32>(0.5)) * 10.0 - t);
    let k = 0.5 + 0.25 * v;
    return vec4<f32>(0.9 * k * k, 0.15 * k, 0.5 + 0.5 * k, 1.0);
}
