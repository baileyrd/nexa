use crate::{
    asset::{load_skeleton_debug_geometry, load_static_geometry, AssetReport, StaticVertex},
    control::{InspectorPanel, RuntimeControls},
};
use anyhow::Context;
use std::time::Instant;
use wgpu::SurfaceError;
use winit::{
    event::{ElementState, Event, KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

pub fn run(report: AssetReport) -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    // A wgpu surface borrows its native window. The viewer is process-lifetime for
    // this small debug executable, so a single owned window is intentionally kept
    // until process exit. Production hosts should own both through their app shell.
    let window: &'static winit::window::Window = Box::leak(Box::new(
        WindowBuilder::new()
            .with_title(title(&report, &RuntimeControls::default()))
            .build(&event_loop)?,
    ));
    let geometry = load_static_geometry(&report.source)?;
    let skeleton = load_skeleton_debug_geometry(&report.source)?;
    let mut controls = RuntimeControls::default();
    if let Some(bounds) = geometry.bounds() {
        controls.camera.frame_bounds(bounds);
    }
    let mut viewer = pollster::block_on(Viewer::new(window, &geometry, &skeleton))?;
    let mut last_update = Instant::now();
    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => target.exit(),
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => viewer.resize(size.width, size.height),
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                let zoom_delta = match delta {
                    MouseScrollDelta::LineDelta(_, vertical) => -vertical * 0.15,
                    MouseScrollDelta::PixelDelta(position) => -(position.y as f32) * 0.002,
                };
                controls.camera.zoom(zoom_delta);
                window.set_title(&title(&report, &controls));
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(key),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                match key {
                    KeyCode::Escape => target.exit(),
                    KeyCode::Digit1 => controls.panel = InspectorPanel::Skeleton,
                    KeyCode::Digit2 => controls.panel = InspectorPanel::MorphTargets,
                    KeyCode::KeyM => controls.select_next_morph(report.morph_target_count),
                    KeyCode::KeyZ => controls.adjust_morph_weight(-0.1),
                    KeyCode::KeyX => controls.adjust_morph_weight(0.1),
                    KeyCode::Digit3 => controls.panel = InspectorPanel::Animation,
                    KeyCode::KeyN => controls.select_next_animation(report.animations.len()),
                    KeyCode::Space => controls.animation_playing = !controls.animation_playing,
                    KeyCode::BracketLeft => controls.scrub(-0.1),
                    KeyCode::BracketRight => controls.scrub(0.1),
                    KeyCode::KeyG => controls.gaze_enabled = !controls.gaze_enabled,
                    KeyCode::KeyV => controls.trigger_viseme("A"),
                    KeyCode::KeyA => controls.nudge_gaze_target(glam::Vec3::new(-0.05, 0.0, 0.0)),
                    KeyCode::KeyD => controls.nudge_gaze_target(glam::Vec3::new(0.05, 0.0, 0.0)),
                    KeyCode::KeyW => controls.nudge_gaze_target(glam::Vec3::new(0.0, 0.0, -0.05)),
                    KeyCode::KeyS => controls.nudge_gaze_target(glam::Vec3::new(0.0, 0.0, 0.05)),
                    KeyCode::KeyQ => controls.nudge_gaze_target(glam::Vec3::new(0.0, 0.05, 0.0)),
                    KeyCode::KeyE => controls.nudge_gaze_target(glam::Vec3::new(0.0, -0.05, 0.0)),
                    KeyCode::ArrowLeft => controls.camera.orbit(-0.08, 0.0),
                    KeyCode::ArrowRight => controls.camera.orbit(0.08, 0.0),
                    KeyCode::ArrowUp => controls.camera.orbit(0.0, 0.08),
                    KeyCode::ArrowDown => controls.camera.orbit(0.0, -0.08),
                    KeyCode::KeyR => controls = RuntimeControls::default(),
                    _ => {}
                }
                window.set_title(&title(&report, &controls));
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                if let Err(e) = viewer.render(
                    &controls.camera,
                    controls.panel == InspectorPanel::Skeleton,
                    controls.selected_morph,
                    controls.morph_weight,
                ) {
                    if !matches!(e, SurfaceError::Outdated | SurfaceError::Lost) {
                        eprintln!("render error: {e:?}");
                    }
                }
            }
            Event::AboutToWait => {
                let now = Instant::now();
                controls.advance((now - last_update).as_secs_f32());
                last_update = now;
                window.set_title(&title(&report, &controls));
                window.request_redraw();
            }
            _ => {}
        }
    })?;
    Ok(())
}

fn title(report: &AssetReport, c: &RuntimeControls) -> String {
    let inspector_detail = match c.panel {
        InspectorPanel::Skeleton => report
            .skins
            .first()
            .map(|skin| format!("{} ({} joints)", skin.name, skin.joint_count))
            .unwrap_or_else(|| "no skeleton".to_owned()),
        InspectorPanel::MorphTargets => report
            .morph_targets
            .get(c.selected_morph)
            .map(|target| format!("{} @ {:.0}%", target.id, c.morph_weight * 100.0))
            .unwrap_or_else(|| "no morph targets".to_owned()),
        InspectorPanel::Animation => report
            .animations
            .get(c.selected_animation)
            .map(|animation| {
                format!(
                    "{} @ {:.2}s{}",
                    animation.name,
                    c.animation_time_seconds,
                    if c.animation_playing {
                        " (playing)"
                    } else {
                        " (paused)"
                    }
                )
            })
            .unwrap_or_else(|| "no animations".to_owned()),
        InspectorPanel::Gaze => format!(
            "{} target [{:.2}, {:.2}, {:.2}]",
            if c.gaze_enabled {
                "enabled"
            } else {
                "disabled"
            },
            c.gaze_target.x,
            c.gaze_target.y,
            c.gaze_target.z
        ),
        _ => format!("t={:.1}s", c.animation_time_seconds),
    };
    format!(
        "Nexa 3D Runtime | {:?}: {} | skins:{} morphs:{} animations:{}",
        c.panel,
        inspector_detail,
        report.skins.len(),
        report.morph_target_count,
        report.animations.len()
    )
}

fn create_skeleton_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Nexa skeleton debug pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<StaticVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 12,
                        shader_location: 1,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 24,
                        shader_location: 2,
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: Default::default(),
        multiview: None,
    })
}

struct Viewer<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    skeleton_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    base_vertices: Vec<StaticVertex>,
    morph_position_deltas: Vec<Vec<[f32; 3]>>,
    active_morph: Option<(usize, u32)>,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    skeleton_vertex_buffer: Option<wgpu::Buffer>,
    skeleton_vertex_count: u32,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}
impl<'a> Viewer<'a> {
    async fn new(
        window: &'a winit::window::Window,
        geometry: &crate::asset::StaticGeometry,
        skeleton: &[StaticVertex],
    ) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("no compatible wgpu adapter")?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Nexa validation device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;
        let size = window.inner_size();
        let format = surface.get_capabilities(&adapter).formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        use wgpu::util::DeviceExt;
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Nexa camera"),
            contents: bytemuck::cast_slice(&[[0.0_f32; 16]]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Nexa camera layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Nexa camera bind group"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Nexa static mesh shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("static_mesh.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Nexa static pipeline layout"),
            bind_group_layouts: &[&camera_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Nexa static mesh pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<StaticVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 12,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 24,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        });
        let skeleton_pipeline =
            create_skeleton_pipeline(&device, &pipeline_layout, &shader, format);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Nexa vertices"),
            contents: bytemuck::cast_slice(&geometry.vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Nexa indices"),
            contents: bytemuck::cast_slice(&geometry.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let skeleton_vertex_buffer = (!skeleton.is_empty()).then(|| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Nexa skeleton debug vertices"),
                contents: bytemuck::cast_slice(skeleton),
                usage: wgpu::BufferUsages::VERTEX,
            })
        });
        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            skeleton_pipeline,
            vertex_buffer,
            base_vertices: geometry.vertices.clone(),
            morph_position_deltas: geometry.morph_position_deltas.clone(),
            active_morph: None,
            index_buffer,
            index_count: geometry.indices.len() as u32,
            skeleton_vertex_buffer,
            skeleton_vertex_count: skeleton.len() as u32,
            camera_buffer,
            camera_bind_group,
        })
    }
    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
    }
    fn render(
        &mut self,
        camera: &crate::control::OrbitCamera,
        show_skeleton: bool,
        selected_morph: usize,
        morph_weight: f32,
    ) -> Result<(), SurfaceError> {
        self.update_morph(selected_morph, morph_weight);
        let aspect = self.config.width as f32 / self.config.height as f32;
        let eye = camera.target
            + glam::Vec3::new(
                camera.yaw_radians.sin() * camera.distance_m,
                camera.pitch_radians.sin() * camera.distance_m + 0.9,
                camera.yaw_radians.cos() * camera.distance_m,
            );
        let view_projection =
            glam::Mat4::perspective_rh_gl(50_f32.to_radians(), aspect, 0.01, 100.0)
                * glam::Mat4::look_at_rh(eye, camera.target, glam::Vec3::Y);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[view_projection.to_cols_array()]),
        );
        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Nexa validation clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.035,
                            g: 0.025,
                            b: 0.065,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            _pass.set_pipeline(&self.pipeline);
            _pass.set_bind_group(0, &self.camera_bind_group, &[]);
            _pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            _pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            _pass.draw_indexed(0..self.index_count, 0, 0..1);
            if show_skeleton {
                if let Some(skeleton) = &self.skeleton_vertex_buffer {
                    _pass.set_pipeline(&self.skeleton_pipeline);
                    _pass.set_vertex_buffer(0, skeleton.slice(..));
                    _pass.draw(0..self.skeleton_vertex_count, 0..1);
                }
            }
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
    fn update_morph(&mut self, selected_morph: usize, weight: f32) {
        let state = (selected_morph, weight.to_bits());
        if self.active_morph == Some(state) {
            return;
        }
        let mut vertices = self.base_vertices.clone();
        if let Some(deltas) = self.morph_position_deltas.get(selected_morph) {
            for (vertex, delta) in vertices.iter_mut().zip(deltas) {
                vertex.position = (glam::Vec3::from(vertex.position)
                    + glam::Vec3::from(*delta) * weight)
                    .to_array();
            }
        }
        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        self.active_morph = Some(state);
    }
}
