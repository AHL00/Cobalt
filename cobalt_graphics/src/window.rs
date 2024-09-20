use winit::{dpi::PhysicalSize, window::WindowAttributes};

pub struct Window {
    winit: winit::window::Window,
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
        event_loop: &winit::event_loop::ActiveEventLoop,
        config: &WindowConfig,
    ) -> Result<Window, Box<dyn std::error::Error>>;

    fn winit(&self) -> &winit::window::Window;

    fn winit_mut(&mut self) -> &mut winit::window::Window;
}

impl WindowInternal for Window {
    fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        config: &WindowConfig,
    ) -> Result<Window, Box<dyn std::error::Error>> {
        let window = event_loop.create_window(
            WindowAttributes::default()
                .with_title(config.title.clone())
                .with_inner_size(winit::dpi::LogicalSize::new(config.size.0, config.size.1)),
        )?;
        Ok(Window { winit: window })
    }

    fn winit(&self) -> &winit::window::Window {
        &self.winit
    }

    fn winit_mut(&mut self) -> &mut winit::window::Window {
        &mut self.winit
    }
}

#[derive(Debug, Clone)]
pub enum Fullscreen {
    Windowed,
    Fullscreen,
    Borderless,
}

impl Window {
    pub fn size(&self) -> (u32, u32) {
        let size = self.winit.inner_size();
        (size.width, size.height)
    }

    pub fn set_min_size(&mut self, size: (u32, u32)) {
        self.winit
            .set_min_inner_size(Some(PhysicalSize::new(size.0, size.1)));
    }

    pub fn set_max_size(&mut self, size: (u32, u32)) {
        self.winit
            .set_max_inner_size(Some(PhysicalSize::new(size.0, size.1)));
    }

    pub fn set_resizable(&mut self, resizable: bool) {
        self.winit.set_resizable(resizable);
    }

    pub fn resizeable(&self) -> bool {
        self.winit.is_resizable()
    }

    pub fn title(&self) -> String {
        self.winit.title()
    }

    pub fn set_title(&mut self, title: &str) {
        self.winit.set_title(title);
    }

    // TODO: Implement set_fullscreen exclusive mode and monitor selection on borderless mode.
    pub fn set_fullscreen(
        &mut self,
        fullscreen: Fullscreen,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match fullscreen {
            Fullscreen::Windowed => {
                self.winit.set_fullscreen(None);
            }
            // Fullscreen::Fullscreen => {
            //     self.winit.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(
            //         VideoMode::primary_monitor().unwrap(),
            //     )));
            // }
            Fullscreen::Borderless => {
                self.winit
                    .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            }
            _ => {
                log::warn!("Set fullscreen mode not implemented yet.");
            }
        }

        Ok(())
    }

    pub fn fullscreen(&self) -> Fullscreen {
        match self.winit.fullscreen() {
            Some(fullscreen) => match fullscreen {
                winit::window::Fullscreen::Exclusive(_) => Fullscreen::Fullscreen,
                winit::window::Fullscreen::Borderless(_) => Fullscreen::Borderless,
            },
            None => Fullscreen::Windowed,
        }
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.winit.set_cursor_visible(visible);
    }
}
