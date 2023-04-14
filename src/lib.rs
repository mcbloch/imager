use std::future::Future;
use clap::Parser;

pub mod cube;
pub mod framework;
pub mod shader_toy;
pub mod util;

pub enum Action {
    UpdateArgs(Args),
    Stop,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value_t = 0)]
    x_pos: u32,
    #[arg(short, long, default_value_t = 0)]
    y_pos: u32,

    #[arg(short, long, default_value_t = 64)]
    width: u32,
    #[arg(long, default_value_t = 64)]
    height: u32,

    #[arg(short, long, default_value_t = false)]
    single: bool,
    // output: Option<String>,
}

pub struct Spawner<'a> {
    executor: async_executor::LocalExecutor<'a>,
}

impl<'a> Spawner<'a> {
    fn new() -> Self {
        Self {
            executor: async_executor::LocalExecutor::new(),
        }
    }

    #[allow(dead_code)]
    pub fn spawn_local(&self, future: impl Future<Output = ()> + 'a) {
        self.executor.spawn(future).detach();
    }

    fn run_until_stalled(&self) {
        while self.executor.try_tick() {}
    }
}

pub trait Renderable: 'static + Sized {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    fn required_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    fn required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
        wgpu::DownlevelCapabilities {
            flags: wgpu::DownlevelFlags::empty(),
            shader_model: wgpu::ShaderModel::Sm5,
            ..wgpu::DownlevelCapabilities::default()
        }
    }
    fn required_limits() -> wgpu::Limits {
        wgpu::Limits::downlevel_webgl2_defaults() // These downlevel limits will allow the code to run on all possible hardware
    }
    fn init(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self;

    fn update(
        &mut self,
        accum_time: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        spawner: &Spawner,
    ) {
        let _ = (accum_time, device, queue, spawner);
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        spawner: &Spawner,
    );
}
