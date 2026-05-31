pub fn error(message: &str) {
    println!("\x1b[31;1m  [ERROR]:\x1b[0m {}", message)
}

pub fn success(message: &str) {
    println!("\x1b[32;1m[SUCCESS]:\x1b[0m {}", message)
}

pub fn info(message: &str) {
    println!("\x1b[36;1m   [INFO]:\x1b[0m {}", message)
}
