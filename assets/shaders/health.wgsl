struct HealthMaterial {
    color_empty: vec4<f32>,
    color_full: vec4<f32>,
    hp: f32,
    hp_max: f32
};

@group(1) @binding(0)
var<uniform> material: HealthMaterial;

fn lerp_color(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32> {
    return (1.0 - t)*a + t*b;
}

@fragment
fn fragment(
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let t = material.hp / material.hp_max;
    if uv.x > t {
        return vec4<f32>(0.0,0.0,0.0,1.0);
    }
    return lerp_color(material.color_empty, material.color_full, t);
}
