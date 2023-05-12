use std::env;
use std::process::exit;
mod config;
use bitcoin::network::get_active_nodes_from_dns_seed;
use bitcoin::config::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config: Config = match Config::from(&args) {
        Err(e) => {
            println!("Application error: {e}");
            exit(1)
        }
        Ok(config) => config,
    };
    let active_nodes =
        match get_active_nodes_from_dns_seed(&config) {
            Err(e) => {
                println!("ERROR: {}", e);
                exit(-1)
            }
            Ok(active_nodes) => active_nodes,
        };
    println!("{:?}", active_nodes);
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_archivo_configuracion() {}
}
