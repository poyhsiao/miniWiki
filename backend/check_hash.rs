use std::io::{self, BufRead};

fn main() {
    let password = std::env::var("CHECK_PASSWORD").ok();
    let hash = std::env::var("CHECK_HASH").ok();

    let (password, hash) = match (password, hash) {
        (Some(p), Some(h)) => {
            let p = p.trim().to_string();
            let h = h.trim().to_string();

            if p.is_empty() || h.is_empty() {
                eprintln!("Error: Both password and hash are required.");
                std::process::exit(1);
            }

            (p, h)
        }
        _ => {
            let stdin = io::stdin();

            println!("Please enter password:");
            let mut password = String::new();
            stdin.lock().read_line(&mut password).expect("Failed to read password from stdin");
            let password = password.trim().to_string();

            println!("Please enter hash:");
            let mut hash = String::new();
            stdin.lock().read_line(&mut hash).expect("Failed to read hash from stdin");
            let hash = hash.trim().to_string();

            if password.is_empty() || hash.is_empty() {
                eprintln!("Error: Both password and hash are required.");
                std::process::exit(1);
            }

            (password, hash)
        }
    };

    match bcrypt::verify(&password, &hash) {
        Ok(true) => {
            println!("Verification successful");
            std::process::exit(0);
        }
        Ok(false) => {
            eprintln!("Verification failed: Password does not match hash");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error during verification: {}", e);
            std::process::exit(1);
        }
    }
}
