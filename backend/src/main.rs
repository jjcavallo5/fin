use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    name: String,
    #[arg(short, long)]
    count: u8,
}

fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        println!("Hello, {}", args.name)
    }
}
