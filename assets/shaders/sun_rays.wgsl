@group(1) @binding(0)
var<uniform> material_time: f32;

@group(1) @binding(1)
var<uniform> material_sun_position: vec4<f32>;

@group(1) @binding(2)
var<uniform> material_sun_color: vec4<f32>;

@group(1) @binding(3)
var<uniform> material_sun_radius: f32;

@group(1) @binding(4)
var<uniform> material_sun_intensity: f32;

struct FragmentInput {
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

fn hash(n: f32) -> f32 {
    return fract(sin(n) * 43758.5453123);
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let sun_pos = material_sun_position.xy;
    let frag_pos = in.world_position.xy;
    let dist = length(frag_pos - sun_pos);

    let base_falloff = clamp(1.0 - dist / material_sun_radius, 0.0, 1.0);
    let glow = pow(base_falloff, 2.2) * material_sun_intensity;
    let edge = smoothstep(material_sun_radius * 0.92, material_sun_radius, dist);
    let color = material_sun_color.rgb * glow * (1.0 - edge);
    let alpha = clamp(glow * 0.85 * (1.0 - edge), 0.0, 0.95);

    return vec4(color, alpha);
}
