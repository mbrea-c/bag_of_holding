use std::sync::Arc;

use bevy::prelude::*;
use zusammen_plugin::ZusammenPlugin;

use crate::protocol::ProtocolPlugin;

pub struct SharedPlugin {
    pub zusammen: Arc<dyn ZusammenPlugin + Send + Sync + 'static>,
}

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin {
            zusammen: self.zusammen.clone(),
        });
        self.zusammen.add_shared(app);
    }
}
