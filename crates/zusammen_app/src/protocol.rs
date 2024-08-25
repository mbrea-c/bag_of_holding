use std::sync::Arc;

use bevy::prelude::*;
use zusammen_plugin::ZusammenPlugin;

pub struct ProtocolPlugin {
    pub zusammen: Arc<dyn ZusammenPlugin + Send + Sync + 'static>,
}

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        self.zusammen.add_client(app);
    }
}
