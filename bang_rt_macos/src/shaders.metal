#include <metal_stdlib>
#include <simd/simd.h>
using namespace metal;

struct VertexIn {
    float2 pos [[attribute(0)]];
};

struct VertexOut {
    float4 pos [[position]];
    float2 uv [[center_no_perspective]];
};

struct Globals {
    uint frame;
    uint _pad;
    float2 reso;
};

vertex VertexOut vertexShader(
    VertexIn in [[stage_in]],
    constant Globals &globals [[buffer(1)]],
    const device float2 *instancePositions [[buffer(2)]],
    unsigned int instanceID [[instance_id]],
    unsigned int vertexID [[vertex_id]]
) {
    float2 instancePos = instancePositions[instanceID];
    VertexOut out;

    out.pos = float4(
        (instancePos.x + in.pos.x) / globals.reso.x * 2.0,
        (instancePos.y + in.pos.y) / globals.reso.y * 2.0,
        0, 1);

    float2 uvs[4] = {
        float2(0, 0), // top-left
        float2(0, 1), // bottom-left
        float2(1, 0), // top-right
        float2(1, 1)  // bottom-right
    };
    out.uv = uvs[vertexID];

    return out;
}


fragment half4 fragmentShader(
    VertexOut in [[stage_in]],
    texture2d<ushort, access::read> tex [[texture(0)]],
    texture1d<half, access::read> pal [[texture(1)]]
) {
    float2 tex_size = float2(tex.get_width(), tex.get_height());
    ushort idx = tex.read(uint2(in.uv * tex_size)).r;
    return pal.read(idx);
}
