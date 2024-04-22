use cobalt::runtime::plugins::{Plugin, PluginError};


pub(crate) struct TestPlugin {
    
}

impl TestPlugin {
    pub fn new() -> Self {
        TestPlugin {}
    }
}

impl Plugin for TestPlugin {
    fn startup(&mut self, _engine: &mut cobalt::runtime::engine::Engine) -> Result<(), cobalt::runtime::plugins::PluginError> {
        log::info!("TestPlugin started!");
        Err(PluginError::NonFatal(String::from("Fake error").into()))
    }

    fn update(&mut self, _engine: &mut cobalt::runtime::engine::Engine)  -> Result<(), PluginError> {
        log::info!("TestPlugin updated!");
        Err(PluginError::Fatal(String::from("Fake error").into()))
    }

    fn name(&self) -> &'static str {
        "TestPlugin"
    }
}