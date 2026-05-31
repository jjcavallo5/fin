use std::{
    io::{self, stdout, Read, Write},
    process::Command,
};

pub fn tui(list: Vec<String>) {
    for item in &list {
        println!("{}", item);
    }
    println!("Press q to quit");
    print!("\x1b[{}A", list.len() + 1);
    io::stdout().flush().unwrap();

    Command::new("stty")
        .args(["raw", "-echo"])
        .status()
        .unwrap();

    let mut selected = 0;
    let result = (|| -> io::Result<()> {
        loop {
            let mut buf = [0u8; 1];
            io::stdin().read_exact(&mut buf)?;

            match buf[0] {
                b'j' => {
                    if selected != list.len() - 1 {
                        // move cursor down one line
                        print!("\x1b[1B\r");
                        stdout().flush().unwrap();
                        selected += 1;
                    }
                }
                b'k' => {
                    if selected != 0 {
                        // move cursor up one line
                        print!("\x1b[1A\r");
                        stdout().flush().unwrap();
                        selected -= 1;
                    }
                }
                b'q' => {
                    print!("\x1b[{}B\r", list.len() - selected);
                    stdout().flush().unwrap();
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    })();

    let _ = Command::new("stty").args(["-raw", "echo"]).status();
}
