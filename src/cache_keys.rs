use crate::{Config, TaskType};
use crossbeam_channel::Sender as CbSender;
use serenity::{
    model::{guild::Role, id::UserId},
    prelude::*,
};

pub struct AdminRolesKey;

impl TypeMapKey for AdminRolesKey {
    type Value = Vec<Role>;
}

pub struct BotIdKey;

impl TypeMapKey for BotIdKey {
    type Value = UserId;
}

pub struct ConfigKey;

impl TypeMapKey for ConfigKey {
    type Value = Config;
}

pub struct GulagRoleKey;

impl TypeMapKey for GulagRoleKey {
    type Value = Role;
}

pub struct HigherRolesKey;

impl TypeMapKey for HigherRolesKey {
    type Value = Vec<Role>;
}

pub struct NitroRoleKey;

impl TypeMapKey for NitroRoleKey {
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
    type Value = CbSender<TaskType>;
}
