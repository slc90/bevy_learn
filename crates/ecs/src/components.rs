use bevy::{
    ecs::{component::Component, entity::Entity, resource::Resource},
    math::Vec2,
};

#[derive(Component, Debug)]
pub struct Player {
    pub name: String,
}

#[derive(Component, Debug)]
pub struct Health(pub i32);

#[derive(Component, Debug, Default)]
pub struct Velocity(pub Vec2);

#[derive(Resource, Default)]
pub struct DemoState {
    pub player: Option<Entity>,
    pub help_printed: bool,
}
