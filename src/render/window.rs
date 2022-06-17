use crate::render::main_state::State;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};



pub async fn run() {
    let event_loop = EventLoop::new(); //Создали постоянный цикл обработки
    let builder = WindowBuilder::new(); //Создали билдер окна
    let window = builder.build(&event_loop).unwrap(); //Создали окно и привязали к окну наш эвент луп и анврапим результат

    let mut state = State::new(&window).await;

    event_loop.run(move |event, _, control_flow| {
        //Запустили эвент луп. Далее вся логика внутри
        *control_flow = ControlFlow::Poll; //Обработчик событий в режиме полинга. Есть вариант Wait

        match event {
            //Матчим ВСЕ эвенты, которые может нам быдать winit
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                // Эвент - Закрытие окна
                *control_flow = ControlFlow::Exit; //Меняем состояние обработчика на состояние ВЫХОДА
            }

            Event::MainEventsCleared => {
                //Здесть ререндерится постоянно. Есть RedrawRequested(_) - тут ререндер по запросу.

                window.request_redraw();
            }

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                //Здесь рендер по запросу и с проверкой окна
                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => {
                        eprintln!("{:?}", e)
                    }
                }
            }

            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == window.id() => {
                //Эвент - закрытие окна
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::Resized(physical_size) => state.resize(*physical_size),
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size)
                        }

                        _ => {} //Конец матчинга event для state
                    }
                }
            }
            _ => (), //Конец матчинга winit
        }
    })
}