#include <metal_stdlib>
#include <simd/simd.h>
using namespace metal;

struct VertexIn {
    float2 pos [[attribute(0)]];
};

struct VertexOut {
    float4 color;
    float4 pos [[position]];
};

struct Globals {
    uint frame;
    uint _pad;
    float2 reso;
};

struct DebugOut {
    VertexOut value;
};

vertex VertexOut vertexShader(
    VertexIn in [[stage_in]],
    constant Globals &globals [[buffer(1)]],
    const device float2 *instancePositions [[buffer(2)]],
    unsigned int instanceID [[instance_id]],
    unsigned int vertexID [[vertex_id]],
    device DebugOut *debugOutputs [[buffer(3)]]
) {
    float2 instancePos = instancePositions[instanceID];
    VertexOut out;

    float phase = globals.frame % 100 / 100.0;

    out.color = float4(1, phase, 0, 1);

    out.pos = float4(
        instancePos.x / globals.reso.x * 2.0 + in.pos.x/5.0,
        instancePos.y / globals.reso.y * 2.0 + in.pos.y/5.0,
        0, 1);

    return out;
}


fragment float4 fragmentShader(VertexOut interpolated [[stage_in]])
{
    return interpolated.color;
}
