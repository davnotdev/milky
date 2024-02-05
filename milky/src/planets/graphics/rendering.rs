#![allow(clippy::needless_update)]

use super::*;
use mepeyew::*;

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct SceneUniform {
    view: glm::Mat4,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct SpriteUniform {
    model: glm::Mat4,
    color: glm::Vec3,
    visible: f32,
}

//  WIP: design rendering flow
#[derive(Clone)]
pub struct RenderSprites {
    pub transforms: Vec<Transform2>,
    pub sprites: Vec<Sprite>,
}

pub struct RenderingPlanet {
    context: Context,
    program: ProgramId,
    compiled_pass: CompiledPassId,
    output_attachment: PassLocalAttachment,

    render_sprites_ev: Ev<RenderSprites>,
    render_sprites_recv: Recv<RenderSprites>,

    scene_ubo: UniformBufferId,
    scene_ubo_guard: UniformBufferTypeGuard<SceneUniform>,
    sprite_ubo: DynamicUniformBufferId,
    sprite_ubo_guard: DynamicUniformBufferTypeGuard<SpriteUniform>,
}

//  TODO: check implications to this
unsafe impl Send for RenderingPlanet {}
unsafe impl Sync for RenderingPlanet {}

impl RenderingPlanet {
    pub fn new(window: &WindowPlanet) -> Self {
        let (width, height) = window.get_size();

        let mut extensions = Extensions::new();
        extensions
            .native_debug(Default::default())
            .naga_translation()
            .surface_extension(SurfaceConfiguration {
                width,
                height,
                display: window.get_raw_display_handle(),
                window: window.get_raw_window_handle(),
            });

        let mut context = Context::new(extensions, None).unwrap();

        let vs = include_bytes!("shaders/vs.spv");
        let fs = include_bytes!("shaders/fs.spv");

        let vs = context
            .naga_translate_shader_code(
                naga_translation::NagaTranslationStage::Vertex,
                naga_translation::NagaTranslationInput::Spirv,
                vs,
                Default::default(),
            )
            .unwrap();
        let fs = context
            .naga_translate_shader_code(
                naga_translation::NagaTranslationStage::Fragment,
                naga_translation::NagaTranslationInput::Spirv,
                fs,
                Default::default(),
            )
            .unwrap();

        let (scene_ubo, scene_ubo_guard) = context
            .new_uniform_buffer(&SceneUniform::default(), None)
            .unwrap();

        let (sprite_ubo, sprite_ubo_guard) = context
            .new_dynamic_uniform_buffer(&[SpriteUniform::default(); 64], None)
            .unwrap();

        let program = context
            .new_program(
                &ShaderSet::shaders(&[
                    (
                        ShaderType::Vertex(VertexBufferInput { args: vec![2, 2] }),
                        &vs,
                    ),
                    (ShaderType::Fragment, &fs),
                ]),
                &[
                    ShaderUniform {
                        set: 0,
                        binding: 0,
                        ty: ShaderUniformType::UniformBuffer(scene_ubo),
                    },
                    ShaderUniform {
                        set: 1,
                        binding: 0,
                        ty: ShaderUniformType::DynamicUniformBuffer(sprite_ubo),
                    },
                ],
                None,
            )
            .unwrap();

        let vbo = context
            .new_vertex_buffer(
                quad_vertices(),
                BufferStorageType::Static,
                Default::default(),
            )
            .unwrap();
        let ibo = context
            .new_index_buffer(
                quad_indices(),
                BufferStorageType::Static,
                Default::default(),
            )
            .unwrap();

        let mut pass = Pass::new(
            width,
            height,
            Some(NewPassExt {
                depends_on_surface_size: Some(()),
                surface_attachment_load_op: Some(PassInputLoadOpColorType::Clear),
                ..Default::default()
            }),
        );
        let output_attachment = pass.get_surface_local_attachment();
        {
            let pass_step = pass.add_step();
            pass_step
                .add_vertex_buffer(vbo)
                .set_index_buffer(ibo)
                .add_program(program)
                .add_write_color(output_attachment);
        }

        let compiled_pass = context.compile_pass(&pass, None).unwrap();

        let render_sprites_ev = Ev::new();

        RenderingPlanet {
            context,
            program,
            compiled_pass,
            output_attachment,
            render_sprites_recv: render_sprites_ev.take(),
            render_sprites_ev,

            scene_ubo,
            scene_ubo_guard,
            sprite_ubo,
            sprite_ubo_guard,
        }
    }

    pub fn get_ev(&self) -> &Ev<RenderSprites> {
        &self.render_sprites_ev
    }

    pub fn update(&mut self, window: &WindowPlanet) {
        let mut submit = Submit::new();
        let mut pass_submit = PassSubmitData::new(self.compiled_pass);

        let sprite_uniforms = self
            .render_sprites_recv
            .iter()
            .flat_map(|ev| {
                ev.transforms
                    .iter()
                    .cloned()
                    .zip(ev.sprites.iter().cloned())
                    .collect::<Vec<_>>()
            })
            .map(|(transform, sprite)| {
                let model = glm::identity();
                let model = glm::translate(
                    &model,
                    &glm::vec3(transform.position.x, transform.position.y, 0.0),
                );
                let model = glm::rotate(&model, transform.rotation, &glm::vec3(0.0, 0.0, 1.0));
                let model = glm::scale(
                    &model,
                    &glm::vec3(transform.scale.x, transform.scale.y, 1.0),
                );

                SpriteUniform {
                    visible: if sprite.visible { 1.0 } else { 0.0 },
                    color: sprite.color,
                    model,
                }
            })
            .collect::<Vec<_>>();
        sprite_uniforms
            .iter()
            .enumerate()
            .for_each(|(idx, obj_data)| {
                submit.transfer_into_dynamic_uniform_buffer(self.sprite_ubo_guard, obj_data, idx);
            });

        let view = glm::translate(&glm::identity(), &glm::vec3(0.0, 0.0, 0.0));

        let scene = SceneUniform { view };
        submit.transfer_into_uniform_buffer(self.scene_ubo_guard, &scene);

        {
            let mut step_submit = StepSubmitData::new();
            for dyn_idx in 0..sprite_uniforms.len() {
                step_submit
                    .draw_indexed(self.program, 0, quad_indices().len())
                    .set_dynamic_uniform_buffer_index(self.sprite_ubo, dyn_idx);
            }
            pass_submit.step(step_submit);
        }
        pass_submit.set_attachment_clear_color(
            self.output_attachment,
            ClearColor {
                r: 0.05,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        );

        submit.pass(pass_submit);
        self.context.submit(submit, None).unwrap();
    }
}
