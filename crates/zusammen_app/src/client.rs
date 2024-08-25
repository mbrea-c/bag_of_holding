use std::sync::Arc;

use bevy::prelude::*;
use zusammen_plugin::ZusammenPlugin;

pub struct ClientPlugin {
    pub zusammen: Arc<dyn ZusammenPlugin + Send + Sync + 'static>,
}

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        self.zusammen.add_client(app);
    }
}
