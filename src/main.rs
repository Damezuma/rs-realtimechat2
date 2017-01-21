#[macro_use(object)]
extern crate json;

mod server;
fn main() {
    let mut server = server::manager::Manager::new().unwrap();
    server.run();

}
