#import bevy_ui::ui_vertex_output::UiVertexOutput

struct AtlasTiled {
    offset: vec2<f32>,
    size: vec2<f32>,
    repeat: vec2<f32>,
}

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;
@group(1) @binding(2)
var<uniform> tiled: AtlasTiled;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;

    // Map the UV coord to tile area
    uv = fract(uv * tiled.repeat) * tiled.size + tiled.offset;

    let color = textureSample(texture, texture_sampler, uv);

    return color;
}
