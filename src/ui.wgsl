// Vertex shader

const tex_coords_array = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 1.0), 
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0)
);
const tex_coords_array_flipped = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 1.0),     
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),    
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 0.0)
);
struct InstanceInput{
    @location(0) location: vec4<f32>,
    @location(1) flip_vertically: u32
}

struct VertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
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
    // let left = -0.5;
    // let right = 0.5;
    // let top = 0.5;
    // let bottom = -0.5;
    var out: VertexOutput;
    let tex_coords = select(tex_coords_array[vertex_index], tex_coords_array_flipped[vertex_index],instance.flip_vertically == 1);
    out.tex_coords = tex_coords;
    // select: false, true, bool
    let x = select(right, left, tex_coords.x == 0.0);
    var y = select(top, bottom, tex_coords.y == 0.0);
    if instance.flip_vertically == 1{
        y = -y;
    }
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}


@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
        let color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
        if (color.a < 0.5){
            discard;
        }
        return color;
}