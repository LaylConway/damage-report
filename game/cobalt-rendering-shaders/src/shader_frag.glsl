#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(set = 0, binding = 1) uniform sampler2D u_sampler_diffuse;

layout(location = 0) in vec2 f_tex_coords;
layout(location = 1) in vec3 f_normal;

layout(location = 0) out vec4 o_color;

// The light is inverse of the angle it should be pointing at
vec3 LIGHT = vec3(0.4, 0.5, 0.6);

void main() {
    // Get the brightness based on the angle between light and normal
    float diffuse_scale = max(dot(normalize(f_normal), normalize(LIGHT)), 0.0f);
    float ambient_scale = 0.05f;
    float total_scale = min(diffuse_scale + ambient_scale, 1.0f);

    // Isolate the diffuse color from the texture
    vec4 tex_color = texture(u_sampler_diffuse, f_tex_coords);
    vec3 diffuse_color = tex_color.rgb;

    // Apply the light strength and alpha
    o_color = vec4(diffuse_color * total_scale, tex_color.a);
}
