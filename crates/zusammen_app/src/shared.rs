use std::sync::Arc;

use bevy::prelude::*;
use zusammen_plugin::ZusammenPlugin;

pub struct SharedPlugin {
    pub zusammen: Arc<dyn ZusammenPlugin + Send + Sync + 'static>,
}

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        self.zusammen.add_shared(app);
    }
}
