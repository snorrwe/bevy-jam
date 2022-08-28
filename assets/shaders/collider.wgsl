struct AABBMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: AABBMaterial;

@fragment
fn fragment(
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    if (uv.x < 0.02 || uv.y < 0.02 || uv.x > 0.98 || uv.y > 0.98) {
        return material.color;
    } else {
        return vec4<f32>(0.0);
    }
}
