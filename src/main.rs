mod text;
mod ui;
mod utils;

use std::error::Error;
use text::{ab_glyph, GlyphBrushBuilder, Section, Text};
use wgpu::util::DeviceExt;
use winit::{event, event_loop};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

fn run_command() -> std::io::Result<String> {
    use std::io::{Write};
    use std::process::{Command, Stdio};
    let mut child = Command::new("ls")
        .current_dir("/Users/hugoamor/Documents/personal/rio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // let child_stdin = child.stdin.as_mut().unwrap();
    // child_stdin.write_all(b"Hello, world!\n")?;
    // Close stdin to finish and avoid indefinite blocking
    // drop(child_stdin);
    
    let output = child.wait_with_output()?;

    // println!("output = {:?}", output);

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-2.0, 1.5, 0.0],
        color: [0.94, 0.47, 0.0],
    }, // A
    Vertex {
        position: [-2.0, 0.83, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // B
    Vertex {
        position: [2.0, 0.83, 0.0],
        color: [0.94, 0.47, 0.0],
    }, // E
    Vertex {
        position: [-2.0, 2.0, 0.0],
        color: [0.8274509804, 0.3176470588, 0.0],
    }, // A
    Vertex {
        position: [-2.0, 0.87, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // B
    Vertex {
        position: [2.0, 0.87, 0.0],
        color: [0.8274509804, 0.3176470588, 0.0],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = event_loop::EventLoop::new();

    let window_builder = utils::create_window_builder("Rio");
    let window = window_builder.build(&event_loop).unwrap();

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };

    let (device, queue) = (async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Request adapter");

        adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Request device")
    })
    .await;

    let mut staging_belt = wgpu::util::StagingBelt::new(1024);
    let render_format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let mut size = window.inner_size();

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: render_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
        },
    );

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices = INDICES.len() as u32;

    let font = ab_glyph::FontArc::try_from_slice(ui::FONT_FIRA_MONO)?;
    let mut glyph_brush =
        GlyphBrushBuilder::using_font(font).build(&device, render_format);

    let command_intro: String = String::from("■ ~ "); // ▲
    let mut command_text: String = String::from("");
    let mut command_result: String = String::from("");
    let mut command_text_y: f32 = 0.0;
    // let mut now_keys = [false; 255];
    // let mut prev_keys = now_keys.clone();

    event_loop.run(move |event, _, control_flow| {
        match event {
            event::Event::WindowEvent {
                event: event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = event_loop::ControlFlow::Exit,

            event::Event::WindowEvent {
                event:
                    event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                match state {
                    winit::event::ElementState::Pressed => {
                        // println!("{:?}", keycode);
                        match keycode {
                            event::VirtualKeyCode::L => {
                                command_text.push_str("l");
                                window.request_redraw();
                            }
                            event::VirtualKeyCode::R => {
                                command_text.push_str("r");
                                window.request_redraw();
                            }
                            event::VirtualKeyCode::I => {
                                command_text.push_str("i");
                                window.request_redraw();
                            }
                            event::VirtualKeyCode::O => {
                                command_text.push_str("o");
                                window.request_redraw();
                            }
                            event::VirtualKeyCode::S => {
                                command_text.push_str("s");
                                window.request_redraw();
                            }
                            event::VirtualKeyCode::Space => {
                                command_text.push_str(" ");
                                window.request_redraw();
                            }
                            event::VirtualKeyCode::Return => {
                                match run_command() {
                                    Ok(result_std) => {
                                        // println!("{:?}", result_std);
                                        command_result = result_std;
                                        window.request_redraw();
                                    }
                                    Err(fail_std) => {
                                        println!("erro: {:?}", fail_std);   
                                    }
                                };


                                // use std::process::Command;
                                // let output = Command::new("vim")
                                //     .arg("/Users/hugoamor/Documents/personal/rio")
                                //     .spawn()
                                //     .expect("failed to execute process");

                                // println!("status: {}", output.status);
                                // println!(
                                //     "stdout: {}",
                                //     String::from_utf8_lossy(&output.stdout)
                                // );
                                // println!(
                                //     "stderr: {}",
                                //     String::from_utf8_lossy(&output.stderr)
                                // );
                            }
                            _ => {
                                println!("code not implemented");
                            }
                        }

                        // window.request_redraw();
                        // now_keys[keycode as usize] = true;
                        // command_text.push_str("a");
                    }
                    winit::event::ElementState::Released => {
                        // now_keys[keycode as usize] = false;
                        // println!("code {:?}", now_keys);
                    }
                }
                // Render only text as typing
            }

            event::Event::WindowEvent {
                event: event::WindowEvent::Resized(new_size),
                ..
            } => {
                size = new_size;

                surface.configure(
                    &device,
                    &wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: render_format,
                        width: size.width,
                        height: size.height,
                        present_mode: wgpu::PresentMode::AutoVsync,
                    },
                );

                window.request_redraw();
            }
            event::Event::RedrawRequested { .. } => {
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Redraw"),
                    });

                let frame = surface.get_current_texture().expect("Get next frame");
                let view = &frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let render_pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Render Pipeline Layout"),
                        bind_group_layouts: &[],
                        push_constant_ranges: &[],
                    });

                let render_pipeline =
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Render Pipeline"),
                        layout: Some(&render_pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: "vs_main",
                            buffers: &[Vertex::desc()],
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: "fs_main",
                            targets: &[Some(wgpu::ColorTargetState {
                                format: render_format,
                                blend: Some(wgpu::BlendState::REPLACE),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: Some(wgpu::Face::Back),
                            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                            polygon_mode: wgpu::PolygonMode::Fill,
                            // Requires Features::DEPTH_CLIP_CONTROL
                            unclipped_depth: false,
                            // Requires Features::CONSERVATIVE_RASTERIZATION
                            conservative: false,
                        },
                        depth_stencil: None, // 1.
                        multisample: wgpu::MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        multiview: None,
                    });

                {
                    let mut render_pass =
                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Clear frame"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(
                                        ui::DEFAULT_COLOR_BACKGROUND,
                                    ),
                                    store: true,
                                },
                            })],
                            depth_stencil_attachment: None,
                        });

                    render_pass.set_pipeline(&render_pipeline); // 2.
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        index_buffer.slice(..),
                        wgpu::IndexFormat::Uint16,
                    );
                    render_pass.draw(0..num_indices, 0..1);
                }

                {
                    glyph_brush.queue(Section {
                        screen_position: (30.0, 120.0),
                        bounds: (size.width as f32, size.height as f32),
                        text: vec![Text::new(&command_intro)
                            .with_color([0.255, 0.191, 0.154, 1.0])
                            .with_scale(36.0)],
                        ..Section::default()
                    });

                    glyph_brush.queue(Section {
                        screen_position: (110.0, 120.0),
                        bounds: (size.width as f32, size.height as f32),
                        text: vec![Text::new(&command_text)
                            .with_color([1.0, 1.0, 1.0, 1.0])
                            .with_scale(36.0)],
                        ..Section::default()
                    });

                    glyph_brush.queue(Section {
                        screen_position: (30.0, 170.0),
                        bounds: (size.width as f32, size.height as f32),
                        text: vec![Text::new(&command_result)
                            .with_color([1.0, 1.0, 1.0, 0.6])
                            .with_scale(36.0)],
                        ..Section::default()
                    });

                    if !command_result.is_empty() {
                        glyph_brush.queue(Section {
                            screen_position: (30.0, 570.0),
                            bounds: (size.width as f32, size.height as f32),
                            text: vec![Text::new(&command_intro)
                                .with_color([0.255, 0.191, 0.154, 1.0])
                                .with_scale(36.0)],
                            ..Section::default()
                        });

                        glyph_brush.queue(Section {
                            screen_position: (110.0, 570.0),
                            bounds: (size.width as f32, size.height as f32),
                            text: vec![Text::new("")
                                .with_color([1.0, 1.0, 1.0, 1.0])
                                .with_scale(36.0)],
                            ..Section::default()
                        });
                    }

                    glyph_brush
                        .draw_queued(
                            &device,
                            &mut staging_belt,
                            &mut encoder,
                            view,
                            size.width,
                            size.height,
                        )
                        .expect("Draw queued");
                }

                staging_belt.finish();
                queue.submit(Some(encoder.finish()));
                frame.present();

                // Recall unused staging buffers
                staging_belt.recall();
            }
            _ => {
                *control_flow = event_loop::ControlFlow::Wait;
            }
        }
    })
}
