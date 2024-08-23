use bevy::prelude::*;

pub trait ZusammenPlugin {
    fn add_protocol(&self, app: &mut App);
    fn add_shared(&self, app: &mut App);
    fn add_server(&self, app: &mut App);
    fn add_client(&self, app: &mut App);
}
