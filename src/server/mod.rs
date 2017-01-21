
pub mod manager;
pub mod message;
pub mod room;
pub mod user;
#[derive(Debug)]
pub enum ChatServerErr{
    FailedInitializeServer
}