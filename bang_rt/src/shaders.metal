#include <metal_stdlib>
#include <simd/simd.h>
using namespace metal;

struct Vertex {
    vector_float4 color;
    vector_float2 pos;
};

struct VertexOut {
    float4 color;
    float4 pos [[position]];
};

vertex VertexOut vertexShader(
    const device Vertex *vertexArray [[buffer(0)]],
    constant float &phase [[buffer(1)]],
    unsigned int vid [[vertex_id]]
) {
    Vertex in = vertexArray[vid];
    VertexOut out;

    out.color = in.color;
    out.pos = float4(in.pos.x+phase, in.pos.y+phase, 0, 1);

    return out;
}

fragment float4 fragmentShader(VertexOut interpolated [[stage_in]])
{
    return interpolated.color;
}
