#version 460

layout(location = 0) in vec2 a_position;

layout(set = 0, binding = 0) uniform Scene {
    mat4 view;
};

layout(set = 1, binding = 0) uniform Sprite {
    mat4 model;
    vec3 color;
    float visible;
};


void main() {
    gl_Position = view * model * vec4(a_position, 0.0, visible);
}
