use egui::epaint::{ImageDelta, Primitive};
use egui::{ClippedPrimitive, Mesh, PaintCallbackInfo, Rect, TextureId};
use metal::*;
use std::ffi::c_void;
use std::mem;

const SHADER_SOURCE: &str = r#"
#include <metal_stdlib>
using namespace metal;

struct VertexIn {
    float2 position [[attribute(0)]];
    float2 uv [[attribute(1)]];
    uchar4 color [[attribute(2)]];
};

struct VertexOut {
    float4 position [[position]];
    float2 uv;
    float4 color;
};

struct Uniforms {
    float2 screen_size;
};

vertex VertexOut vertex_main(VertexIn in [[stage_in]], constant Uniforms &uniforms [[buffer(1)]]) {
    VertexOut out;
    out.position = float4(
        2.0 * in.position.x / uniforms.screen_size.x - 1.0,
        1.0 - 2.0 * in.position.y / uniforms.screen_size.y,
        0.0,
        1.0
    );
    out.uv = in.uv;
    out.color = float4(in.color) / 255.0;
    return out;
}

fragment float4 fragment_main(VertexOut in [[stage_in]], texture2d<float> texture [[texture(0)]], sampler textureSampler [[sampler(0)]]) {
    return in.color * texture.sample(textureSampler, in.uv);
}
"#;

pub struct MetalRenderer {
    device: Device,
    pipeline_state: RenderPipelineState,
    command_queue: CommandQueue,
    texture: Option<Texture>,
    sampler: SamplerState,
}

impl MetalRenderer {
    pub fn new(device: Device) -> Self {
        let library = device
            .new_library_with_source(SHADER_SOURCE, &CompileOptions::new())
            .expect("Failed to compile shader");

        let vertex_function = library
            .get_function("vertex_main", None)
            .expect("Vertex function not found");
        let fragment_function = library
            .get_function("fragment_main", None)
            .expect("Fragment function not found");

        let descriptor = RenderPipelineDescriptor::new();
        descriptor.set_vertex_function(Some(&vertex_function));
        descriptor.set_fragment_function(Some(&fragment_function));

        let color_attachment = descriptor.color_attachments().object_at(0).unwrap();
        color_attachment.set_pixel_format(MTLPixelFormat::BGRA8Unorm);

        color_attachment.set_blending_enabled(true);
        color_attachment.set_rgb_blend_operation(MTLBlendOperation::Add);
        color_attachment.set_alpha_blend_operation(MTLBlendOperation::Add);
        color_attachment.set_source_rgb_blend_factor(MTLBlendFactor::One);
        color_attachment.set_destination_rgb_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);
        color_attachment.set_source_alpha_blend_factor(MTLBlendFactor::OneMinusDestinationAlpha);
        color_attachment.set_destination_alpha_blend_factor(MTLBlendFactor::One);

        let vertex_descriptor = VertexDescriptor::new();

        let attr0 = vertex_descriptor.attributes().object_at(0).unwrap();
        attr0.set_format(MTLVertexFormat::Float2);
        attr0.set_offset(0);
        attr0.set_buffer_index(0);

        let attr1 = vertex_descriptor.attributes().object_at(1).unwrap();
        attr1.set_format(MTLVertexFormat::Float2);
        attr1.set_offset(8);
        attr1.set_buffer_index(0);

        let attr2 = vertex_descriptor.attributes().object_at(2).unwrap();
        attr2.set_format(MTLVertexFormat::UChar4);
        attr2.set_offset(16);
        attr2.set_buffer_index(0);

        let layout = vertex_descriptor.layouts().object_at(0).unwrap();
        layout.set_stride(20);
        descriptor.set_vertex_descriptor(Some(vertex_descriptor));

        let pipeline_state = device
            .new_render_pipeline_state(&descriptor)
            .expect("Failed to create pipeline state");

        let command_queue = device.new_command_queue();

        let sampler_descriptor = SamplerDescriptor::new();
        sampler_descriptor.set_min_filter(MTLSamplerMinMagFilter::Linear);
        sampler_descriptor.set_mag_filter(MTLSamplerMinMagFilter::Linear);
        let sampler = device.new_sampler(&sampler_descriptor);

        Self {
            device,
            pipeline_state,
            command_queue,
            texture: None,
            sampler,
        }
    }

    pub fn update_texture(&mut self, id: TextureId, delta: &ImageDelta) {
        if let Some(pos) = delta.pos {
            // Partial update
            if let Some(texture) = &self.texture {
                match &delta.image {
                    egui::ImageData::Color(image) => {
                        let region = MTLRegion {
                            origin: MTLOrigin {
                                x: pos[0] as u64,
                                y: pos[1] as u64,
                                z: 0,
                            },
                            size: MTLSize {
                                width: image.width() as u64,
                                height: image.height() as u64,
                                depth: 1,
                            },
                        };
                        let bytes: Vec<u8> =
                            image.pixels.iter().flat_map(|p| p.to_array()).collect();
                        texture.replace_region(
                            region,
                            0,
                            bytes.as_ptr() as *const c_void,
                            (image.width() * 4) as u64,
                        );
                    }
                    egui::ImageData::Font(image) => {
                        let region = MTLRegion {
                            origin: MTLOrigin {
                                x: pos[0] as u64,
                                y: pos[1] as u64,
                                z: 0,
                            },
                            size: MTLSize {
                                width: image.width() as u64,
                                height: image.height() as u64,
                                depth: 1,
                            },
                        };
                        let bytes: Vec<u8> = image
                            .srgba_pixels(None)
                            .flat_map(|p| p.to_array())
                            .collect();
                        texture.replace_region(
                            region,
                            0,
                            bytes.as_ptr() as *const c_void,
                            (image.width() * 4) as u64,
                        );
                    }
                }
            }
        } else {
            // Full update / new texture
            match &delta.image {
                egui::ImageData::Color(image) => {
                    let descriptor = TextureDescriptor::new();
                    descriptor.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
                    descriptor.set_width(image.width() as u64);
                    descriptor.set_height(image.height() as u64);
                    let texture = self.device.new_texture(&descriptor);
                    let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                    let region = MTLRegion {
                        origin: MTLOrigin { x: 0, y: 0, z: 0 },
                        size: MTLSize {
                            width: image.width() as u64,
                            height: image.height() as u64,
                            depth: 1,
                        },
                    };
                    texture.replace_region(
                        region,
                        0,
                        bytes.as_ptr() as *const c_void,
                        (image.width() * 4) as u64,
                    );
                    self.texture = Some(texture);
                }
                egui::ImageData::Font(image) => {
                    let descriptor = TextureDescriptor::new();
                    descriptor.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
                    descriptor.set_width(image.width() as u64);
                    descriptor.set_height(image.height() as u64);
                    let texture = self.device.new_texture(&descriptor);
                    // Conversion
                    let bytes: Vec<u8> = image
                        .srgba_pixels(None)
                        .flat_map(|p| p.to_array())
                        .collect();
                    let region = MTLRegion {
                        origin: MTLOrigin { x: 0, y: 0, z: 0 },
                        size: MTLSize {
                            width: image.width() as u64,
                            height: image.height() as u64,
                            depth: 1,
                        },
                    };
                    texture.replace_region(
                        region,
                        0,
                        bytes.as_ptr() as *const c_void,
                        (image.width() * 4) as u64,
                    );
                    self.texture = Some(texture);
                }
            }
        }
    }

    pub fn render(
        &mut self,
        drawable: &TextureRef,
        primitives: Vec<ClippedPrimitive>,
        screen_size: (f32, f32),
        pixels_per_point: f32,
    ) {
        let command_buffer = self.command_queue.new_command_buffer();

        let pass_descriptor = RenderPassDescriptor::new();
        let color_attachment = pass_descriptor.color_attachments().object_at(0).unwrap();
        color_attachment.set_texture(Some(drawable));
        color_attachment.set_load_action(MTLLoadAction::Load);
        color_attachment.set_store_action(MTLStoreAction::Store);

        let encoder = command_buffer.new_render_command_encoder(&pass_descriptor);
        encoder.set_render_pipeline_state(&self.pipeline_state);

        let uniforms = [
            screen_size.0 / pixels_per_point,
            screen_size.1 / pixels_per_point,
        ];
        encoder.set_vertex_bytes(1, 8, uniforms.as_ptr() as *const c_void);

        if let Some(texture) = &self.texture {
            encoder.set_fragment_texture(0, Some(texture));
            encoder.set_fragment_sampler_state(0, Some(&self.sampler));
        }

        encoder.set_viewport(MTLViewport {
            originX: 0.0,
            originY: 0.0,
            width: screen_size.0 as f64,
            height: screen_size.1 as f64,
            znear: 0.0,
            zfar: 1.0,
        });

        for primitive in primitives {
            if let Primitive::Mesh(mesh) = primitive.primitive {
                if mesh.vertices.is_empty() {
                    continue;
                }

                // Dynamic buffer allocation (inefficient for large meshes but safe for now)
                let v_bytes: &[u8] = unsafe {
                    std::slice::from_raw_parts(
                        mesh.vertices.as_ptr() as *const u8,
                        mesh.vertices.len() * 20,
                    )
                };
                let v_buffer = self.device.new_buffer_with_data(
                    v_bytes.as_ptr() as *const c_void,
                    v_bytes.len() as u64,
                    MTLResourceOptions::CPUCacheModeDefaultCache,
                );

                let i_bytes: &[u8] = unsafe {
                    std::slice::from_raw_parts(
                        mesh.indices.as_ptr() as *const u8,
                        mesh.indices.len() * 4,
                    )
                };
                let i_buffer = self.device.new_buffer_with_data(
                    i_bytes.as_ptr() as *const c_void,
                    i_bytes.len() as u64,
                    MTLResourceOptions::CPUCacheModeDefaultCache,
                );

                encoder.set_vertex_buffer(0, Some(&v_buffer), 0);

                // Clip rect
                let clip_rect = primitive.clip_rect;
                // Need to convert logical clip_rect to viewport pixels
                let clip_min_x = (clip_rect.min.x * pixels_per_point).round() as u64;
                let clip_min_y = (clip_rect.min.y * pixels_per_point).round() as u64;
                let clip_max_x = (clip_rect.max.x * pixels_per_point).round() as u64; // Correct: ScissorRect is [x, y, w, h]
                let clip_w = (clip_max_x - clip_min_x).max(0);
                let clip_h =
                    ((clip_rect.max.y * pixels_per_point).round() as u64 - clip_min_y).max(0);

                // Clamp
                let target_w = drawable.width();
                let target_h = drawable.height();

                if clip_min_x < target_w && clip_min_y < target_h {
                    encoder.set_scissor_rect(MTLScissorRect {
                        x: clip_min_x,
                        y: clip_min_y,
                        width: clip_w.min(target_w - clip_min_x),
                        height: clip_h.min(target_h - clip_min_y),
                    });

                    encoder.draw_indexed_primitives(
                        MTLPrimitiveType::Triangle,
                        mesh.indices.len() as u64,
                        MTLIndexType::UInt32,
                        &i_buffer,
                        0,
                    );
                }
            }
        }

        encoder.end_encoding();
        command_buffer.commit();
        command_buffer.wait_until_scheduled(); // Optional, but usually good for UI overlays
    }
}
