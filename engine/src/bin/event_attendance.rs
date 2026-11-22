//! event_attendance <data dir> <request.json>
#[path = "../attendance.rs"]
mod attendance;
use std::path::Path;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let data = Path::new(&args[1]);
    let request: attendance::AttendanceRequest =
        serde_json::from_reader(std::fs::File::open(&args[2]).unwrap()).unwrap();
    if let Some(profile) = &request.audience_profile {
        let cuts = csv::Reader::from_path(data.join("communes.csv")).unwrap().records()
            .map(|row| row.unwrap()[0].to_owned()).collect::<Vec<_>>();
        assert!(profile.valid(&cuts), "Expected a source and valid origin shares for known comuna CUTs.");
    }
    let result = attendance::project(request, false);
    serde_json::to_writer(std::io::stdout().lock(), &result).unwrap();
}
