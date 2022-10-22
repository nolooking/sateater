use std::process::Command;

pub fn cashu_receive(token: &str) -> bool {
    let output = Command::new("cashu")
        .arg("receive")
        .arg(token)
        .output()
        .expect("failed to run cashu receive");

    let output_str = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = output_str.split("\n").collect();

    if lines[1] == "" {
        return false;
    }

    let balance1 = lines[0].split(" ").collect::<Vec<&str>>()[1];
    let balance2 = lines[1].split(" ").collect::<Vec<&str>>()[1];
    dbg!(balance1, balance2);
    // Always paid if gets to this stage!
    // This will need to change but makes debugging easy for now with a single cashu instance.
    true
}
