use crate::ZusammenPlugin;
use bevy::prelude::*;
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct CombinedPlugins {
    plugins: Vec<Arc<dyn ZusammenPlugin + Send + Sync + 'static>>,
}

impl CombinedPlugins {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn and<C: ZusammenPlugin + Send + Sync + 'static>(mut self, c: C) -> Self {
        self.plugins.push(Arc::new(c));
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
