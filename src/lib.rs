use wallet::Wallet;

pub mod account;
pub mod address_decoder;
pub mod blocks;
pub mod compact_size_uint;
pub mod config;
pub mod gtk;
pub mod handler;
pub mod handshake;
pub mod initial_block_download;
pub mod logwriter;
pub mod messages;
pub mod network;
pub mod node;
pub mod transactions;
pub mod utxo_tuple;
pub mod wallet;


pub fn terminal_ui(mut wallet: Wallet) {
    show_options();
    loop {
        let mut input = String::new();

        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                println!("\n");
                let command = input.trim();
                if let Ok(num) = command.parse::<u32>() {
                    match num {
                        0 => {
                            println!("Cerrando nodo...\n");
                            break;
                        }
                        1 => {
                            handle_add_account_request(&mut wallet);
                        }
                        2 => {
                            handle_balance_request(&mut wallet);
                        }
                        3 => {
                            handle_transaccion_request(&mut wallet);
                        }
                        _ => {
                            println!("Número no reconocido. Inténtalo de nuevo! \n");
                        }
                    }
                    show_options();
                } else {
                    println!("Entrada inválida. Inténtalo de nuevo! \n");
                }
            }
            Err(error) => {
                println!("Error al leer la entrada: {}", error);
            }
        }
    }

}

fn show_options() {
    println!("\n");
    println!("INGRESE ALGUNO DE LOS SIGUIENTES COMANDOS\n");
    println!("0: Terminar el programa");
    println!("1: Añadir una cuenta a la wallet");
    println!("2: Mostrar balance de las cuentas");
    println!("3: Hacer transaccion desde una cuenta");
    println!("4: Prueba de inclusion de una transaccion en un bloque");
    println!("-----------------------------------------------------------\n");
}

fn handle_transaccion_request(wallet: &mut Wallet) {
    if wallet.show_indexes_of_accounts().is_none() {
        return;
    }
    println!("INGRESE LOS SIGUIENTES DATOS PARA REALIZAR UNA TRANSACCION \n");
    let account_index: usize = read_input("Índice de la cuenta: ").unwrap_or_else(|err| {
        println!("Error al leer la entrada: {}", err);
        0
    });
    let address_receiver: String = read_input("Dirección del receptor: ").unwrap_or_else(|err| {
        println!("Error al leer la entrada: {}", err);
        String::new()
    });
    let amount: i64 = read_input("Cantidad(Satoshis): ").unwrap_or_else(|err| {
        println!("Error al leer la entrada: {}", err);
        0
    });
    let fee: i64 = read_input("Tarifa(Satoshis): ").unwrap_or_else(|err| {
        println!("Error al leer la entrada: {}", err);
        0
    });
    println!("Realizando y broadcasteando transaccion...");
    if let Err(error) = wallet.make_transaction(account_index, &address_receiver, amount, fee) {
        println!("Error al realizar la transacción: {}", error);
    } else {
        println!("TRANSACCION REALIZADA CORRECTAMENTE!");
    }
}

fn read_input<T: std::str::FromStr>(prompt: &str) -> Result<T, std::io::Error>
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    println!("{}", prompt);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let value: T = input.trim().parse().map_err(|err| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Error parsing input: {}", err))
    })?;
    Ok(value)
}

fn handle_add_account_request(wallet: &mut Wallet)  {
    println!("Ingrese PRIVATE KEY en formato WIF: ");
    let mut private_key_input = String::new();
    match std::io::stdin().read_line(&mut private_key_input) {
        Ok(_) => {
            let wif_private_key = private_key_input.trim();
            println!("Ingrese la ADDRESS de la cuenta: ");
            let mut address_input = String::new();
            match std::io::stdin().read_line(&mut address_input) {
                Ok(_) => {
                    let address = address_input.trim();
                    println!("Agregando la cuenta -- {} -- a la wallet...\n", address);
                    if let Err(err) = wallet.add_account(wif_private_key.to_string(), address.to_string()) {
                        println!("ERROR: {err}\n");
                        println!("Ocurrio un error al intentar añadir una nueva cuenta, intente de nuevo! \n");
                    } else {
                        println!("CUENTA -- {} -- AÑADIDA CORRECTAMENTE A LA WALLET!\n", address);
                    }
                }
                Err(error) => {
                    println!("Error al leer la entrada: {}", error);
                }
            }
        }
        Err(error) => {
            println!("Error al leer la entrada: {}", error);
        }
    }

}



fn handle_balance_request(wallet: &mut Wallet) {
    println!("Calculando el balance de las cuentas...\n");
    wallet.show_accounts_balance();
}


