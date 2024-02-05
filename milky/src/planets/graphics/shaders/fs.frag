#version 460

layout(location = 0) out vec4 o_color;

layout(set = 1, binding = 0) uniform Sprite {
    mat4 model;
    vec3 color;
    float visible;
};

void main() {
    o_color = vec4(color, 1.0);
}
