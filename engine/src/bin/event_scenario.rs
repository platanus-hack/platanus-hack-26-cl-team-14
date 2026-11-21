//! Manual run: cargo run --release --manifest-path engine/Cargo.toml --bin event_scenario -- data/mvp '<request json>'
#[path = "../event.rs"]
mod event;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let m = event::EventModel::new(std::path::Path::new(&a[1]));
    match m.scenario(serde_json::from_str(&a[2]).unwrap()) { Ok(v) => println!("{}", serde_json::to_string(&v).unwrap()), Err(e) => println!("422 {e}") }
}
