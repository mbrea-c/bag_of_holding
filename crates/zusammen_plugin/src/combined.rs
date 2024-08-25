use crate::ZusammenPlugin;
use bevy::prelude::*;

#[derive(Default, Clone)]
pub struct CombinedPlugins {
    plugins: Vec<Box<dyn ZusammenPlugin + Clone + Send + Sync + 'static>>,
}

impl CombinedPlugins {
    pub fn new(plugins: impl IntoIterator<Item = Box<dyn ZusammenPlugin>>) -> Self {
        Self {
            plugins: plugins.into_iter().collect(),
        }
    }

    pub fn and<C: ZusammenPlugin + 'static>(mut self, c: C) -> Self {
        self.plugins.push(Box::new(c));
        self
    }
}

impl ZusammenPlugin for CombinedPlugins {
    fn add_protocol(&self, app: &mut App) {
        for plug in &self.plugins {
            plug.add_protocol(app);
        }
    }

    fn add_shared(&self, app: &mut App) {
        for plug in &self.plugins {
            plug.add_shared(app);
        }
    }

    fn add_server(&self, app: &mut App) {
        for plug in &self.plugins {
            plug.add_server(app);
        }
    }

    fn add_client(&self, app: &mut App) {
        for plug in &self.plugins {
            plug.add_client(app);
        }
    }
}
