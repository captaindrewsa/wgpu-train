use wgpu_train1::render;

fn main() {
    pollster::block_on(render::window::run());
}
