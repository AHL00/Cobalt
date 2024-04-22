pub struct Window {
    // TODO: Maybe make this private and expose own methods?
    pub winit: winit::window::Window,
}

#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub size: (u32, u32),
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig {
            title: "Cobalt Engine".to_string(),
            size: (800, 600),
        }
    }
}

pub trait WindowInternal {
    fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
        config: WindowConfig,
    ) -> Result<Window, Box<dyn std::error::Error>>;
}

impl WindowInternal for Window {
    fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
        config: WindowConfig,
    ) -> Result<Window, Box<dyn std::error::Error>> {
        let window = winit::window::WindowBuilder::new()
            .with_title(config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(config.size.0, config.size.1))
            .build(event_loop)?;
        Ok(Window { winit: window })
    }
}

impl Window {}
