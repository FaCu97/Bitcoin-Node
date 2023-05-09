use std::env;
use std::process::exit;
mod config;
use bitcoin::network::get_active_nodes_from_dns_seed;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = config::Config::from(&args);

    if let Err(e) = config {
        println!("Application error: {e}");
        exit(1);
    }
    let active_nodes =
        match get_active_nodes_from_dns_seed("seed.testnet.bitcoin.sprovoost.nl".to_string()) {
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
