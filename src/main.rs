use clap::{Arg, Command};
use inquire::Select;
use regex::Regex;

fn cli() -> Command {
    Command::new("sniff")
        .about("CLI for listing services listening on a port and killing it")
        .arg(
            Arg::new("port")
                .help("The port to sniff")
                .required(true)
                .index(1),
        )
}

fn main() {
    let matches = cli().get_matches();

    let port_str = matches.get_one::<String>("port").expect("Port is required");

    match port_str.parse::<usize>() {
        Ok(port) => {
            let output = std::process::Command::new("lsof")
                .arg("-i")
                .arg(format!("TCP:{}", port))
                .output()
                .expect("Failed to execute command");

            let output_str = String::from_utf8_lossy(&output.stdout);

            let re = Regex::new(r"^(?P<command>\S+)\s+(?P<pid>\d+)")
                .expect("Invalid regex. Could not parse output of lsof command.");

            let mut choices: Vec<String> = Vec::new();

            for line in output_str.lines() {
                if let Some(caps) = re.captures(line) {
                    println!("Process: {:<15} PID: {}", &caps["command"], &caps["pid"]);

                    choices.push(format!(
                        "Process: {:<15} PID: {}",
                        &caps["command"], &caps["pid"]
                    ));
                }
            }

            let ans = Select::new("Choose a process to kill", choices).prompt();

            match ans {
                Ok(choice) => {
                    println!("You chose: {}", choice);

                    std::process::Command::new("kill")
                        .arg("-9")
                        .arg(&choice.split(" ").last().expect("Invalid choice"))
                        .output()
                        .expect("Failed to execute command");
                }
                Err(_) => println!("There was an error, please try again"),
            }
        }
        Err(_) => {
            eprintln!("Error: Port must be a valid number");
            std::process::exit(1);
        }
    }
}
