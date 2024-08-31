use std::{collections::VecDeque, marker::PhantomData, time::Instant};

use bevy::{
    diagnostic::DiagnosticsStore, ecs::component::ComponentId, prelude::*, ptr::Ptr, utils::HashMap,
};
use bevy_inspector_egui::egui;
use lightyear::{
    client::prediction::diagnostics::PredictionMetrics, transport::io::IoDiagnosticsPlugin,
};
use serde::{Deserialize, Serialize};

use crate::plugin::{ClientMarker, LocalDebugModule};

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct CharacterDebugData<M: Send + Sync> {
    rollback_triggers: HashMap<(Entity, Option<String>), u32>,
    rollbacks: u32,
    rollback_ticks: u32,
    bytes_in: f64,
    #[serde(skip)]
    bytes_out: VecDeque<(Instant, f64)>,
    #[serde(skip)]
    #[serde(default = "Instant::now")]
    time: Instant,
    __phantom_data: PhantomData<M>,
}

impl<M: Send + Sync> Default for CharacterDebugData<M> {
    fn default() -> Self {
        Self {
            rollback_triggers: HashMap::default(),
            rollbacks: 0,
            rollback_ticks: 0,
            bytes_in: 0.,
            bytes_out: VecDeque::default(),
            time: Instant::now(),
            __phantom_data: PhantomData::default(),
        }
    }
}

pub fn update_data<M: Send + Sync + 'static>(
    mut debug_data: ResMut<CharacterDebugData<M>>,
    prediction_metrics: Res<PredictionMetrics>,
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time<Real>>,
) {
    debug_data.rollbacks = prediction_metrics.rollbacks;
    debug_data.rollback_ticks = prediction_metrics.rollback_ticks;
    debug_data.bytes_in = diagnostics
        .get(&IoDiagnosticsPlugin::BYTES_IN)
        .and_then(|d| d.value())
        .unwrap_or(0.);

    if let Some(d) = diagnostics
        .get(&IoDiagnosticsPlugin::BYTES_OUT)
        .and_then(|d| d.measurement())
        .map(|m| (m.time, m.value))
    {
        debug_data.bytes_out.push_back(d);
    }

    while debug_data.bytes_out.len() > 1600 {
        debug_data.bytes_out.pop_front();
    }

    debug_data.time = time.last_update().unwrap_or(Instant::now());
}

pub struct ClientLightyearDebugModule;

impl ClientLightyearDebugModule {
    fn insert_data<M: Default + Send + Sync + 'static>(&self, app: &mut App) -> ComponentId {
        app.world_mut().init_resource::<CharacterDebugData<M>>()
    }

    fn add_update_systems<M: Default + Send + Sync + 'static>(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, update_data::<M>);
    }

    fn render_ui<M: Default + Send + Sync + 'static>(&self, ui: &mut egui::Ui, data: Ptr) {
        let data = unsafe { data.deref::<CharacterDebugData<M>>() };
        let mut main_rollback_triggers = data.rollback_triggers.iter().collect::<Vec<_>>();
        ui.heading("Top rollback causes:");
        main_rollback_triggers.sort_by_key(|(_, v)| -(**v as i32));
        main_rollback_triggers
            .into_iter()
            .take(10)
            .map(|((entity, maybe_name), val)| {
                format!(
                    "\t[{}:{}-{}] -> {}",
                    entity.index(),
                    entity.generation(),
                    match maybe_name {
                        Some(n) => n.split("::").last().unwrap(),
                        None => "",
                    },
                    val
                )
            })
            .for_each(|s| {
                ui.label(s);
            });

        ui.label(format!("Rollbacks: {}", data.rollbacks));
        ui.label(format!("Rollback ticks: {}", data.rollback_ticks));
        // ui.label(format!("Rx: {}", data.bytes_in));

        // plot(ui, &data.bytes_out, data.time);
    }
}

// pub fn plot(ui: &mut egui::Ui, data: &VecDeque<(Instant, f64)>, time: Instant) {
//     let (mut response, painter) =
//         ui.allocate_painter(egui::Vec2::new(50., 50.), egui::Sense::drag());
//
//     let to_screen = egui::emath::RectTransform::from_to(
//         egui::Rect::from_min_size(egui::Pos2::ZERO, response.rect.square_proportions()),
//         response.rect,
//     );
//     let from_screen = to_screen.inverse();
//
//     let max_y = data
//         .iter()
//         .map(|(_, y)| y)
//         .copied()
//         .reduce(|l, r| if l.abs() > r.abs() { l.abs() } else { r.abs() })
//         .unwrap_or(0.);
//
//     let points: Vec<egui::Pos2> = data
//         .iter()
//         .map(|(d, m)| {
//             // Map (Duration, f64) to egui::Pos2
//             //  1. First map Duration to x axis
//             let now = time;
//             let earlier = now.checked_sub(Duration::from_secs(60)).unwrap();
//             let d = d.duration_since(earlier);
//             let x = d.as_secs_f32() / 60.;
//             //  2. Then map f64 to y axis
//             let y = (m / max_y) as f32;
//
//             egui::Pos2::new(x, y)
//         })
//         .collect();
//
//     let points: Vec<egui::Pos2> = points.iter().map(|p| to_screen * *p).collect();
//     let shape = egui::Shape::line(points, PathStroke::new(1., egui::Color32::GREEN));
//
//     painter.add(shape);
// }

impl LocalDebugModule for ClientLightyearDebugModule {
    fn insert_data(&self, app: &mut App) -> ComponentId {
        self.insert_data::<ClientMarker>(app)
    }

    fn add_update_systems(&self, app: &mut App) {
        self.add_update_systems::<ClientMarker>(app)
    }

    fn render_ui(&self, ui: &mut egui::Ui, data: Ptr) {
        self.render_ui::<ClientMarker>(ui, data)
    }
}
