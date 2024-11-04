use std::{future::Future, ops::Fn, pin::Pin};

use futures::future::BoxFuture;
use reqwest::Client;

use super::{
    model::{channel::Channel, user::User},
    Context, ContextSupplier,
};

pub type CommandFn = fn(InteractionContext<'_>, String, User, Channel) -> BoxFuture<'_, ()>;

pub struct InteractionContext<'a> {
    pub(super) ctx: Context<'a>,
    pub trigger_id: String,
    //pub(super)
}

impl<'a> ContextSupplier<'a> for InteractionContext<'a> {
    fn get_context(&self) -> &Context<'a> {
        &self.ctx
    }
}

pub struct SlashCommand {
    pub command: String,
    pub execute: CommandFn,
}

pub type ActionFn = fn(InteractionContext<'_>, User, String, String, Channel) -> BoxFuture<'_, ()>;

pub struct MessageAction {
    pub action: String,
    pub execute: ActionFn,
}
