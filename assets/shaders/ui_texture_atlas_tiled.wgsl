#import bevy_ui::ui_vertex_output::UiVertexOutput

struct AtlasTiled {
    atlas_grids: vec2<f32>,
    tile_size: vec2<f32>,
    tile_pos: vec2<f32>,
}

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;
@group(1) @binding(2)
var<uniform> tiled: AtlasTiled;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // Wrap UV coord to fit the tile range  
    var uv = fract(in.uv * in.size / tiled.tile_size);
    // Map the UV coord to tile area
    uv = (uv + tiled.tile_pos) / tiled.atlas_grids;

    let color = textureSample(texture, texture_sampler, uv);

    return color;
}
