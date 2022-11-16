use std::process::Command;

pub fn cashu_receive(token: &str) -> u32 {
    let output = Command::new("cashu")
        .arg("receive")
        .arg(token)
        .output()
        .expect("failed to run cashu receive");

    let output_str = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = output_str.split("\n").collect();

    if lines[1] == "" {
        return 0;
    }

    let balance1 = lines[0].split(" ").collect::<Vec<&str>>()[1]
        .parse::<u32>()
        .unwrap();
    let balance2 = lines[1].split(" ").collect::<Vec<&str>>()[1]
        .parse::<u32>()
        .unwrap();

    // Always paid if gets to this stage!
    // This will need to change but makes debugging easy for now with a single cashu instance.
    balance2.checked_sub(balance1).expect("ok")
}

pub fn cashu_send(amount: u32) -> String {
    let output = Command::new("cashu")
        .arg("send")
        .arg(amount.to_string())
        .output()
        .expect("failed to run cashu send");

    dbg!("GETTING CASHU OUTPUT!");
    dbg!(&output);
    let output_str = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = output_str.split("\n").collect();
    let token: String = lines[1].to_string();
    assert!(!token.contains(" "));
    token
}
