// Slow flowing aurora ribbons.
@group(0) @binding(0) var<uniform> time: vec4<f32>;
@group(0) @binding(1) var<uniform> rect: vec4<f32>;

@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let t = time.x * 0.4;
    var col = vec3<f32>(0.0, 0.02, 0.05);
    for (var i: i32 = 0; i < 3; i = i + 1) {
        let f = f32(i);
        let y = 0.3 + 0.2 * f + 0.12 * sin(uv.x * (3.0 + f) + t * (1.0 + f * 0.3) + f * 2.0);
        let band = exp(-pow((uv.y - y) * 7.0, 2.0));
        let tint = vec3<f32>(0.1 + 0.3 * f, 0.8 - 0.2 * f, 0.7 + 0.1 * f);
        col = col + tint * band * 0.7;
    }
    return vec4<f32>(col, 1.0);
}
