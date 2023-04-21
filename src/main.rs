use std::env;
use std::process;
mod config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = config::Config::from(&args);

    if let Err(e) = config {
        println!("Application error: {e}");
        process::exit(1);
    }
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_archivo_configuracion() {}
}
