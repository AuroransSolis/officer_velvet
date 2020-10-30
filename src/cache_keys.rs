use crate::{Config, TaskType};
use crossbeam_channel::Sender;
use serenity::{
    model::{guild::Role, id::UserId},
    prelude::*,
};

pub struct BotIdKey;

impl TypeMapKey for BotIdKey {
    type Value = UserId;
}

pub struct ConfigKey;

impl TypeMapKey for ConfigKey {
    type Value = Config;
}

pub struct ElevatedRolesKey;

impl TypeMapKey for ElevatedRolesKey {
    type Value = Vec<Role>;
}

pub struct GulagRoleKey;

impl TypeMapKey for GulagRoleKey {
    type Value = Role;
}

pub struct ReadyKey;

impl TypeMapKey for ReadyKey {
    type Value = bool;
}

pub struct TasksKey;

impl TypeMapKey for TasksKey {
    type Value = Vec<TaskType>;
}

pub struct TaskSenderKey;

impl TypeMapKey for TaskSenderKey {
    type Value = Sender<TaskType>;
}
