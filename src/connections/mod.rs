
mod server;
mod client;

pub use server::Server;
pub use client::Client;

#[derive(Clone, Copy)]
pub enum Action {
    Loading,
    Benchmarking
}

impl Action {
    pub fn as_int(&self) -> i32 {
        match self {
            Action::Loading => 1,
            Action::Benchmarking => 2
        }
    }
}