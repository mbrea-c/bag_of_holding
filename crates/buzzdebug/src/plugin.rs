//! What do we need for a debug module?
//! 1. Data resource
//! 2. System(s) for updating data resource (maybe placed together in plugin)
//! 3. Rendering function showing UI based on data resource i.e. `fn(&mut egui::Ui, &DataRes)`
//!
//! Additionally, if the module is remote, items 1. and 2. would run in the server, and the data
//! resource would need to be replicated onto client. The UI function would obviously run in
//! client

use std::sync::Arc;

use bevy::{ecs::component::ComponentId, prelude::*, ptr::Ptr};
use bevy_inspector_egui::egui;
use zusammen_plugin::ZusammenPlugin;

use super::ui::{debug_menu_system, DebugMenuData};

#[derive(Default)]
pub struct ServerMarker;
#[derive(Default)]
pub struct ClientMarker;

pub trait LocalDebugModule: Send + Sync {
    fn insert_data(&self, app: &mut App) -> ComponentId;
    fn add_update_systems(&self, app: &mut App);
    fn render_ui(&self, ui: &mut egui::Ui, data: Ptr);
}

pub trait RemoteDebugModule: Send + Sync {
    #[allow(unused_variables)]
    fn add_protocol(&self, app: &mut App) {}
    fn insert_data(&self, app: &mut App) -> ComponentId;
    fn add_update_systems(&self, app: &mut App);
    fn render_ui(&self, ui: &mut egui::Ui, data: Ptr);
}

#[derive(Resource, Clone)]
pub struct DebugZusammenPlugin {
    pub locals: Vec<(String, Arc<dyn LocalDebugModule>)>,
    pub remote: Vec<(String, Arc<dyn RemoteDebugModule>)>,
}

#[derive(Clone)]
pub enum DebugMode {
    Server(Arc<dyn RemoteDebugModule>),
    Client(Arc<dyn LocalDebugModule>),
}

#[derive(Clone)]
pub struct ActiveModule {
    pub name: String,
    pub data: ComponentId,
    pub mode: DebugMode,
}

#[derive(Resource, Clone, Default)]
pub struct DebugModules {
    pub modules: Vec<ActiveModule>,
}

impl DebugZusammenPlugin {
    pub fn new() -> Self {
        Self {
            locals: vec![],
            remote: vec![],
        }
    }

    pub fn with_local<C: LocalDebugModule + 'static>(
        mut self,
        name: impl Into<String>,
        module: C,
    ) -> Self {
        self.locals.push((name.into(), Arc::new(module)));
        self
    }

    pub fn with_remote<C: RemoteDebugModule + 'static>(
        mut self,
        name: impl Into<String>,
        module: C,
    ) -> Self {
        self.remote.push((name.into(), Arc::new(module)));
        self
    }
}

impl ZusammenPlugin for DebugZusammenPlugin {
    fn add_protocol(&self, app: &mut App) {
        for (_, module) in &self.remote {
            module.add_protocol(app);
        }
    }

    fn add_shared(&self, _: &mut App) {}

    fn add_server(&self, app: &mut App) {
        for (_, module) in &self.remote {
            module.insert_data(app);
            module.add_update_systems(app);
        }
    }

    fn add_client(&self, app: &mut App) {
        let mut debug_modules = DebugModules::default();
        app.insert_resource(DebugMenuData { should_show: true });
        // This one needs to be in update because it's an UI rendering system
        app.add_systems(Update, debug_menu_system);

        for (name, module) in &self.locals {
            let data_id = module.insert_data(app);
            debug_modules.modules.push(ActiveModule {
                name: name.clone(),
                data: data_id,
                mode: DebugMode::Client(module.clone()),
            });
        }

        for (name, module) in &self.remote {
            let data_id = module.insert_data(app);
            debug_modules.modules.push(ActiveModule {
                name: name.clone(),
                data: data_id,
                mode: DebugMode::Server(module.clone()),
            });
        }

        app.insert_resource(debug_modules);

        for (_, module) in &self.locals {
            module.add_update_systems(app);
        }
    }
}
