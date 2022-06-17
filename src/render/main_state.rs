use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]

struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex{
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>{
        wgpu::VertexBufferLayout{
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute{
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,

                },
                wgpu::VertexAttribute{
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2
                }
            ]
        }
    }
}

// Changed
const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 1.0 - 0.99240386], }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 1.0 - 0.56958647], }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 1.0 - 0.05060294], }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 1.0 - 0.1526709], }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 1.0 - 0.7347359], }, // E
];


const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];
pub struct State {
    surface: wgpu::Surface,                  //Поверхность для отрисовки
    device: wgpu::Device,                    //Определение видеокарты
    queue: wgpu::Queue,                      //Тут запросы?
    pub size: winit::dpi::PhysicalSize<u32>, //физический размер окна
    config: wgpu::SurfaceConfiguration,      //какие-то конфигурации?
    clear_color: wgpu::Color,                //Цвет по дефолту
    render_pipeline: wgpu::RenderPipeline,   //Конвейр для работы с шейдерами
    vertex_buffer: wgpu::Buffer,             //БУФЕР
    num_vertices: u32,                       //Количество вершин
    index_buffer: wgpu::Buffer,              //
    num_indices: u32,     
    diffuse_bind_group : wgpu::BindGroup,                   //
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size(); //Задали размер как у окна сейчас
        let num_vertices = VERTICES.len() as u32; //Сохранили количетсво вершин нашей фигуры
        let instance = wgpu::Instance::new(wgpu::Backends::all()); //Создали инстанс.
        let surface = unsafe { instance.create_surface(window) }; //Создали повернхность для отрисовки в окне
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                //Какие-то общие опции
                power_preference: wgpu::PowerPreference::default(), //Настроили задействованную мощность
                compatible_surface: Some(&surface), //Хз что это. Надо в документацию
                force_fallback_adapter: false,      //Хз что это. Надо в документацию
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    //Создаем какой-то дескриптор
                    features: wgpu::Features::empty(), //Настройка 1 features
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    }, //Настройка 2 limits. Тут условие, зависящее от архитектуры (вдруг webassembly будет)
                    label: None,
                }, //Настройка 3
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let clear_color = wgpu::Color::BLACK;

        surface.configure(&device, &config);
        
        //Работаем с текстурами
        let diffuse_bytes = include_bytes!("images/goblin.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let dimensions = diffuse_rgba.dimensions();
        
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                // Все такстуры сохарняются как 3D. 
                // Поэтому мы выставляем dimension как 2D
                size: texture_size,
                mip_level_count: 1, // мипкарты
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Нужно конвертировать в sRGB
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                // TEXTURE_BINDING  сообщает wgpu, что мы хотим использовать эту текстуру в шейдерах
                // COPY_DST  означает, что мы хотим скопировать данные в эту текстуру
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("diffuse_texture"),
            }
        );

        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &diffuse_rgba,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            texture_size,
        );

        let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        //Создаем группы привязки
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,


                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        let diffuse_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                    },
                    wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                    }
                ],
                    label: Some("diffuse_bind_group"),
            }
        );      

        

        //Далее мы работает с PIPELINE с ШЕЙДЕРАМИ
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        }); // ЗАГРУЗКА ШЕЙДЕРА
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            }); //

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                ],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            multiview: None,
        }); //СДЕЛАЛИ КОНВЕЙР ШЕЙДЕРОВ ДЛЯ ЗАГРУЗКИ
            //ДАЛЕЕ мы работаем с буфером
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX
        });
        let num_indices = INDICES.len() as u32;

        Self {
            surface,
            device,
            queue,
            size,
            config,
            clear_color,
            render_pipeline,
            vertex_buffer,
            num_vertices,
            index_buffer,
            num_indices,
            diffuse_bind_group,

        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: position.x / self.size.width as f64,
                    g: position.y / self.size.height as f64,
                    b: 1.0,
                    a: 1.0,
                };
                true
            }
            WindowEvent::KeyboardInput {input,..} => { 
                match input.virtual_keycode { //Матчим какую именно кнопку нажали
                    Some(winit::event::VirtualKeyCode::Space) =>{ //Если нажат пробел
                        println!("SPACE was pressed");
                    }
                    _ => () //Выходим из матча 
                }
                true
            }
            WindowEvent::KeyboardInput { input, .. }=>{
                match input.virtual_keycode {
                    Some(winit::event::VirtualKeyCode::Left) =>{
                        todo!("Зделать изменение буфера через умножение на вектор")
                    }
                    _=>()
                }
                true
            }
            WindowEvent::KeyboardInput { input, .. }=>{
                match input.virtual_keycode {
                    Some(winit::event::VirtualKeyCode::Right) =>{
                        todo!("Зделать изменение буфера через умножение на вектор")
                    }
                    _=>()
                }
                true
            }
            _=> false
        }
    }
    
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]); // NEW!
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);            
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
    pub fn update(&mut self) {}
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.height = new_size.height;
            self.config.width = new_size.width;
            self.surface.configure(&self.device, &self.config);
        }
    }
}
