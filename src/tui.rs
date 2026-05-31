use std::{
    io::{self, stdout, Read, Write},
    process::Command,
};

fn print_selected(item: &String) {
    print!("\x1B[1m\x1B[32m{}\x1B[0m\r", item);
}

fn reset_line(item: &String) {
    print!("\r\x1b[2K{}", item);
}

fn move_down() {
    print!("\x1b[1B\r");
}

fn move_up() {
    print!("\x1b[1A\r");
}

pub fn tui(list: Vec<String>) -> (String, usize) {
    for (i, item) in list.iter().enumerate() {
        if i == 0 {
            print_selected(item);
            print!("\n");
        } else {
            println!("{}", item);
        }
    }
    println!("Press q to quit");
    print!("\x1b[{}A\x1b[?25l", list.len() + 1); // Moves cursor to start & makes it invisible
    io::stdout().flush().unwrap();

    Command::new("stty")
        .args(["raw", "-echo"])
        .status()
        .unwrap();

    let mut selected = 0;
    let mut should_exit = false;
    let _ = (|| -> io::Result<()> {
        loop {
            let mut buf = [0u8; 1];
            io::stdin().read_exact(&mut buf)?;

            match buf[0] {
                b'j' => {
                    if selected != list.len() - 1 {
                        reset_line(&list[selected]);
                        move_down();
                        selected += 1;
                        print_selected(&list[selected]);
                        stdout().flush().unwrap();
                    }
                }
                b'k' => {
                    if selected != 0 {
                        reset_line(&list[selected]);
                        move_up();
                        selected -= 1;
                        print_selected(&list[selected]);
                        stdout().flush().unwrap();
                    }
                }
                b'\r' | b'\n' => {
                    print!("\x1b[{}B\x1b[?25h\r", list.len() - selected); // Moves cursor to end and
                    stdout().flush().unwrap();
                    break;
                }
                b'q' => {
                    print!("\x1b[{}B\x1b[?25h\r", list.len() - selected); // Moves cursor to end and
                    stdout().flush().unwrap();
                    should_exit = true;
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    })();

    let _ = Command::new("stty").args(["-raw", "echo"]).status();

    if should_exit {
        std::process::exit(1);
    }

    return (list[selected].clone(), selected);
}
