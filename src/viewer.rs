use crate::{
    animation::{load_animation_clip, AnimationClip},
    asset::{
        load_animated_skeleton_debug_geometry, load_skeleton_debug_geometry, load_static_geometry,
        AssetReport, StaticVertex,
    },
    control::{InspectorPanel, RuntimeControls},
    skin::{load_skin_rig, SkinBinding, VertexSkin},
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
    let rig = load_skin_rig(&report.source)?;
    let skin = geometry
        .skin_index
        .and_then(|index| rig.skins.get(index))
        .cloned();
    let animation_clips: Vec<AnimationClip> = (0..report.animations.len())
        .map(|index| load_animation_clip(&report.source, index))
        .collect::<Result<_, _>>()?;
    let mut morph_weights = vec![0.0_f32; report.morph_target_count];
    let mut controls = RuntimeControls::default();
    if let Some(bounds) = geometry.bounds() {
        controls.camera.frame_bounds(bounds);
    }
    let mut viewer = pollster::block_on(Viewer::new(window, &geometry, &skeleton, skin.as_ref()))?;
    // An asset with a skin but no animations never reaches the per-frame palette
    // update, so seed the rest pose here rather than leaving identities behind.
    if let Some(skin) = &skin {
        viewer.update_joints(&skin.joint_matrices(&rig.hierarchy.rest_world_transforms()));
    }
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
                    KeyCode::KeyJ => controls.select_next_node(report.nodes.len()),
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
                    &morph_weights,
                ) {
                    if !matches!(e, SurfaceError::Outdated | SurfaceError::Lost) {
                        eprintln!("render error: {e:?}");
                    }
                }
            }
            Event::AboutToWait => {
                let now = Instant::now();
                let duration = report
                    .animations
                    .get(controls.selected_animation)
                    .map(|animation| animation.duration_seconds)
                    .unwrap_or(0.0);
                controls.advance_looping((now - last_update).as_secs_f32(), duration);
                last_update = now;
                morph_weights = match animation_clips.get(controls.selected_animation) {
                    Some(clip) => {
                        if let Ok(posed_skeleton) = load_animated_skeleton_debug_geometry(
                            &report.source,
                            &clip.transforms,
                            controls.animation_time_seconds,
                        ) {
                            viewer.update_skeleton(&posed_skeleton);
                        }
                        let pose = clip.sample_pose(controls.animation_time_seconds);
                        if let Some(skin) = &skin {
                            let world = rig.hierarchy.world_transforms(&pose);
                            viewer.update_joints(&skin.joint_matrices(&world));
                        }
                        pose.morph_slot_weights(report.morph_target_count)
                    }
                    None => vec![0.0; report.morph_target_count],
                };
                // The manual morph slider stays authoritative for its own target so
                // an unanimated target can still be posed by hand.
                if let Some(manual) = morph_weights.get_mut(controls.selected_morph) {
                    *manual = manual.max(controls.morph_weight);
                }
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
            .nodes
            .get(c.selected_node)
            .map(|node| {
                format!(
                    "{} #{}{}{}",
                    node.name,
                    node.index,
                    if node.is_joint { " (joint)" } else { "" },
                    if node.has_mesh { " (mesh)" } else { "" }
                )
            })
            .unwrap_or_else(|| "no nodes".to_owned()),
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
                    "{} @ {:.2}/{:.2}s{}",
                    animation.name,
                    c.animation_time_seconds,
                    animation.duration_seconds,
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
            entry_point: "vs_skeleton",
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
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 40,
                        shader_location: 3,
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
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24Plus,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview: None,
    })
}

fn create_depth_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("Nexa depth buffer"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

struct Viewer<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    depth_view: wgpu::TextureView,
    pipeline: wgpu::RenderPipeline,
    skeleton_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    skin_vertex_buffer: wgpu::Buffer,
    joint_buffer: wgpu::Buffer,
    joint_bind_group: wgpu::BindGroup,
    joint_capacity: usize,
    base_vertices: Vec<StaticVertex>,
    morph_position_deltas: Vec<Vec<[f32; 3]>>,
    active_morph_weights: Vec<f32>,
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
        skin: Option<&SkinBinding>,
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
        let depth_view = create_depth_view(&device, &config);
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
        // A skinless asset still binds a one-entry identity palette so the
        // pipeline layout, and the shader, need no unskinned variant.
        let joint_capacity = skin.map(SkinBinding::joint_count).unwrap_or(0).max(1);
        let joint_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Nexa joint palette"),
            contents: bytemuck::cast_slice(&vec![
                glam::Mat4::IDENTITY.to_cols_array();
                joint_capacity
            ]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let joint_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Nexa joint palette layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let joint_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Nexa joint palette bind group"),
            layout: &joint_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: joint_buffer.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Nexa static mesh shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("static_mesh.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Nexa static pipeline layout"),
            bind_group_layouts: &[&camera_layout, &joint_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Nexa static mesh pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
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
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 40,
                                shader_location: 3,
                            },
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<VertexSkin>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Uint32x4,
                                offset: 0,
                                shader_location: 4,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 16,
                                shader_location: 5,
                            },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
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
        let skin_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Nexa vertex skin bindings"),
            contents: bytemuck::cast_slice(&geometry.vertex_skins),
            usage: wgpu::BufferUsages::VERTEX,
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
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
        });
        Ok(Self {
            surface,
            device,
            queue,
            config,
            depth_view,
            pipeline,
            skeleton_pipeline,
            vertex_buffer,
            skin_vertex_buffer,
            joint_buffer,
            joint_bind_group,
            joint_capacity,
            base_vertices: geometry.vertices.clone(),
            morph_position_deltas: geometry.morph_position_deltas.clone(),
            active_morph_weights: Vec::new(),
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
        self.depth_view = create_depth_view(&self.device, &self.config);
    }
    fn update_skeleton(&mut self, skeleton: &[StaticVertex]) {
        if skeleton.len() as u32 != self.skeleton_vertex_count {
            return;
        }
        if let Some(buffer) = &self.skeleton_vertex_buffer {
            self.queue
                .write_buffer(buffer, 0, bytemuck::cast_slice(skeleton));
        }
    }
    fn render(
        &mut self,
        camera: &crate::control::OrbitCamera,
        show_skeleton: bool,
        morph_weights: &[f32],
    ) -> Result<(), SurfaceError> {
        self.update_morph(morph_weights);
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            _pass.set_pipeline(&self.pipeline);
            _pass.set_bind_group(0, &self.camera_bind_group, &[]);
            _pass.set_bind_group(1, &self.joint_bind_group, &[]);
            _pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            _pass.set_vertex_buffer(1, self.skin_vertex_buffer.slice(..));
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
    /// Uploads this frame's joint palette. Entries beyond what the buffer was
    /// sized for are dropped rather than reallocating mid-frame; the skin's joint
    /// count is fixed at import.
    fn update_joints(&mut self, matrices: &[glam::Mat4]) {
        let columns: Vec<[f32; 16]> = matrices
            .iter()
            .take(self.joint_capacity)
            .map(glam::Mat4::to_cols_array)
            .collect();
        if columns.is_empty() {
            return;
        }
        self.queue
            .write_buffer(&self.joint_buffer, 0, bytemuck::cast_slice(&columns));
    }

    /// Rebuilds the vertex buffer from every non-zero morph target weight, so an
    /// animated `weights` channel and the manual slider both reach the GPU.
    fn update_morph(&mut self, weights: &[f32]) {
        if self.active_morph_weights == weights {
            return;
        }
        let mut vertices = self.base_vertices.clone();
        for (slot, weight) in weights.iter().enumerate() {
            if *weight == 0.0 {
                continue;
            }
            let Some(deltas) = self.morph_position_deltas.get(slot) else {
                continue;
            };
            for (vertex, delta) in vertices.iter_mut().zip(deltas) {
                vertex.position = (glam::Vec3::from(vertex.position)
                    + glam::Vec3::from(*delta) * *weight)
                    .to_array();
            }
        }
        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        self.active_morph_weights = weights.to_vec();
    }
}

#[cfg(test)]
mod tests {
    /// The viewer needs a GPU to run, so the shader would otherwise only be
    /// checked at `create_shader_module` time on a developer's machine. Naga is
    /// the same front end wgpu uses, so parsing and validating here catches a
    /// broken shader in CI instead.
    #[test]
    fn the_static_mesh_shader_compiles_and_validates() {
        let source = include_str!("static_mesh.wgsl");
        let module = naga::front::wgsl::parse_str(source).expect("shader failed to parse");
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .expect("shader failed validation");
        let entry_points: Vec<&str> = module
            .entry_points
            .iter()
            .map(|entry| entry.name.as_str())
            .collect();
        assert_eq!(entry_points, ["vs_main", "vs_skeleton", "fs_main"]);
    }
}
