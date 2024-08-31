use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiContext, egui};

use super::plugin::{DebugMode, DebugModules};

#[derive(Resource)]
pub struct DebugMenuData {
    pub should_show: bool,
}

pub fn debug_menu_system(world: &mut World) {
    if !world.resource::<DebugMenuData>().should_show {
        return;
    };

    let unsafe_world_cell = world.as_unsafe_world_cell();

    let mut egui_ctx = unsafe { unsafe_world_cell.world_mut() }.query::<&mut EguiContext>();
    let Ok(mut ctx) = egui_ctx.get_single_mut(unsafe { unsafe_world_cell.world_mut() }) else {
        return;
    };

    let debug_modules = unsafe { unsafe_world_cell.world() }.resource::<DebugModules>();

    egui::Window::new("Buzzdebug!").show(ctx.get_mut(), |ui| {
        for module in &debug_modules.modules {
            let data = unsafe { unsafe_world_cell.world() }
                .get_resource_by_id(module.data)
                .unwrap();
            ui.collapsing(&module.name, |ui| {
                match &module.mode {
                    DebugMode::Server(module) => module.render_ui(ui, data),
                    DebugMode::Client(module) => module.render_ui(ui, data),
                };
            });
        }
    });
}
