use rocket::http::Status;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

fn count_request() {
    let logfile = "request.log";
    if !Path::new(logfile).exists() {
        fs::File::create(logfile).expect("created request.log");
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(logfile)
        .unwrap();

    if let Err(e) = writeln!(
        file,
        "{:?}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

#[get("/wildcard")]
pub async fn wildcard() -> (Status, String) {
    let contents = fs::read_to_string("wildcard").expect("could not open wildcard file");

    count_request();
    (Status::Accepted, contents)
}
