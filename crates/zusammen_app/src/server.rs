use std::sync::Arc;

use bevy::prelude::*;
use zusammen_plugin::ZusammenPlugin;

pub struct ServerPlugin {
    pub zusammen: Arc<dyn ZusammenPlugin + Send + Sync + 'static>,
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        self.zusammen.add_server(app);
    }
}
