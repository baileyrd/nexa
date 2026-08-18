struct Camera { view_projection: mat4x4<f32> };
@group(0) @binding(0) var<uniform> camera: Camera;

// Joint palette for the single skinned mesh this debug viewer draws. Entries are
// `jointWorld * inverseBind`, so they map mesh-node-local vertices to scene
// space; an unposed skin uploads identities and renders at its bind pose.
@group(1) @binding(0) var<storage, read> joint_matrices: array<mat4x4<f32>>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) emission: vec3<f32>,
    @location(4) joints: vec4<u32>,
    @location(5) weights: vec4<f32>,
};
struct SkeletonInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) emission: vec3<f32>,
};
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) emission: vec3<f32>,
};

fn skin_matrix(joints: vec4<u32>, weights: vec4<f32>) -> mat4x4<f32> {
    let zero = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var blended = mat4x4<f32>(zero, zero, zero, zero);
    let available = arrayLength(&joint_matrices);
    for (var influence = 0u; influence < 4u; influence = influence + 1u) {
        let weight = weights[influence];
        let joint = joints[influence];
        if (weight != 0.0 && joint < available) {
            blended = blended + joint_matrices[joint] * weight;
        }
    }
    return blended;
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var position = vec4<f32>(input.position, 1.0);
    var normal = input.normal;
    // glTF asks for unit-sum weights; normalizing keeps a sloppy export from
    // shrinking or inflating the mesh.
    let total = input.weights.x + input.weights.y + input.weights.z + input.weights.w;
    if (total > 0.0) {
        let skin = skin_matrix(input.joints, input.weights / total);
        position = skin * position;
        normal = (skin * vec4<f32>(input.normal, 0.0)).xyz;
    }
    var output: VertexOutput;
    output.position = camera.view_projection * position;
    output.normal = normal;
    output.color = input.color;
    output.emission = input.emission;
    return output;
}

@vertex
fn vs_skeleton(input: SkeletonInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = camera.view_projection * vec4<f32>(input.position, 1.0);
    output.normal = input.normal;
    output.color = input.color;
    output.emission = input.emission;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let light = max(dot(normalize(input.normal), normalize(vec3<f32>(0.3, 0.8, 0.5))), 0.18);
    return vec4<f32>(input.color.rgb * light + input.emission, input.color.a);
}
