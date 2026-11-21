//! fans <data dir> <venue CUT> <attendance>: the fixed fan base at one venue, as JSON.
#[allow(dead_code)]
#[path = "../choice.rs"]
mod choice;
#[path = "../fans.rs"]
mod fans;
use std::path::Path;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let dir = Path::new(&a[1]);
    let c = choice::Choice::load(dir);
    let r = fans::Fans::load(dir).project(&c, &c.minutes, &a[2], a[3].parse().unwrap());
    serde_json::to_writer(std::io::stdout().lock(), &r).unwrap();
}
