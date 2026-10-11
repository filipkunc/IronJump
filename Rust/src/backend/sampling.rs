//! One sprite pipeline: linear sampling for the ball, pixel-center sampling for
//! tiles and bitmap glyphs. This keeps filtering differences inside one batch.
use macroquad::{miniquad::*, prelude::*};

pub fn material() -> Material {
    let backend = unsafe {
        macroquad::window::get_internal_gl()
            .quad_context
            .info()
            .backend
    };
    load_material(
        match backend {
            Backend::OpenGl => ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: FRAGMENT,
            },
            Backend::Metal => ShaderSource::Msl { program: METAL },
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("sprite sampling shader")
}

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
attribute vec4 normal;
uniform mat4 Model;
uniform mat4 Projection;
varying highp vec2 uv;
varying lowp vec4 color;
varying highp vec3 sampling;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
    color = color0 / 255.0;
    sampling = normal.xyz;
}"#;

const FRAGMENT: &str = r#"#version 100
precision highp float;
uniform sampler2D Texture;
varying highp vec2 uv;
varying lowp vec4 color;
varying highp vec3 sampling;
void main() {
    vec2 nearest = (floor(uv * sampling.yz) + 0.5) / sampling.yz;
    gl_FragColor = color * texture2D(Texture, mix(nearest, uv, sampling.x));
}"#;

const METAL: &str = r#"
#include <metal_stdlib>
using namespace metal;
struct Vertex {
    float3 position [[attribute(0)]];
    float2 texcoord [[attribute(1)]];
    float4 color0 [[attribute(2)]];
    float4 normal [[attribute(3)]];
};
struct RasterizerData {
    float4 position [[position]];
    float2 uv;
    float4 color;
    float3 sampling;
};
struct Uniforms { float4x4 Model; float4x4 Projection; float4 _Time; };
vertex RasterizerData vertexShader(Vertex v [[stage_in]], constant Uniforms& u [[buffer(0)]]) {
    RasterizerData out;
    out.position = u.Projection * u.Model * float4(v.position, 1.0);
    out.uv = v.texcoord;
    out.color = v.color0 / 255.0;
    out.sampling = v.normal.xyz;
    return out;
}
fragment float4 fragmentShader(RasterizerData in [[stage_in]],
    texture2d<float> Texture [[texture(0)]], sampler TextureSmplr [[sampler(0)]]) {
    float2 nearest = (floor(in.uv * in.sampling.yz) + 0.5) / in.sampling.yz;
    return in.color * Texture.sample(TextureSmplr, mix(nearest, in.uv, in.sampling.x));
}
"#;
