// Vertex shader

const tex_coords_array = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 1.0), 
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0)
);
struct InstanceInput{
    @location(0) location: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) depth: f32,
    @location(3) use_texture: u32,
}

struct VertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) use_texture: u32,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    instance: InstanceInput,
) -> VertexOutput  {
    let left = instance.location.x;
    let top = instance.location.y;
    let right = instance.location.z;
    let bottom = instance.location.w;
    var out: VertexOutput;
    let tex_coords = tex_coords_array[vertex_index];
    out.tex_coords = tex_coords;
    // select: false, true, bool
    let x = select(right, left, tex_coords.x == 0.0);
    let y = select(top, bottom, tex_coords.y == 0.0);
    out.clip_position = vec4<f32>(x, y, instance.depth, 1.0);
    out.color = instance.color;
    out.use_texture = instance.use_texture;
    return out;
}


@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.use_texture == 1u) {
        return textureSample(t_diffuse, s_diffuse, in.tex_coords);
    } else {
        return in.color;
    }
}