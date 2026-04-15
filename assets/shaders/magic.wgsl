@group(1) @binding(0)
var<uniform> material_color: vec4<f32>;

@group(1) @binding(1)
var<uniform> material_time: f32;

@group(1) @binding(2)
var<uniform> material_light_position: vec4<f32>;

@group(1) @binding(3)
var<uniform> material_light_color: vec4<f32>;

@group(1) @binding(4)
var<uniform> material_light_radius: f32;

struct FragmentInput {
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let glow = 0.45 + 0.35 * sin(material_time * 3.5 + in.uv.x * 8.0);
    let base = material_color.rgb * (0.75 + 0.25 * glow);

    let light_vec = material_light_position.xyz - in.world_position.xyz;
    let distance = length(light_vec);
    let light_strength = clamp(1.0 - distance / material_light_radius, 0.0, 1.0);
    let light_diffuse = material_light_color.rgb * light_strength * 1.4;

    let normal = normalize(in.world_normal);
    let light_dir = normalize(light_vec);
    let spec = pow(max(dot(normal, light_dir), 0.0), 20.0) * light_strength * 0.7;

    let edge = smoothstep(0.0, 0.012, abs(in.uv.y - 0.5) * 2.0);
    let color = base + light_diffuse + spec * material_light_color.rgb;
    return vec4(color, 1.0 - edge * 0.25);
}
