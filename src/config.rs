use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::str::FromStr;
pub struct Config {
    pub NUMBER_OF_NODES: usize,
    pub DNS_SEED: String,
    pub TESTNET_PORT: String,
    pub TESTNET_START_STRING: [u8;4],
    pub PROTOCOL_VERSION: i32,
    pub USER_AGENT: String
}
impl Config {
    /// Crea un config leyendo un archivo de configuracion ubicado en la
    ///  ruta especificada en los argumentos recibidos por parametro.
    /// El formato del contenido es: {config_name}={config_value}
    /// Devuelve un Config con los valores leidos del archivo especificado
    ///
    /// Devuelve un io::Error si:
    /// - No se pudo encontrar el archivo en la ruta indicada.
    /// - El archivo tiene un formato invalido.
    pub fn from(args: &[String]) -> Result<Self, Box<dyn Error>> {
        if args.len() > 2 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Too many arguments".to_string(),
            )));
        }

        if args.len() < 2 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Not enough arguments".to_string(),
            )));
        }
        let file = File::open(&args[1])?;
        Self::from_reader(file)
    }

    fn from_reader<T: Read>(content: T) -> Result<Config, Box<dyn Error>> {
        let reader = BufReader::new(content);

        let mut cfg = Self {
            NUMBER_OF_NODES: 0,
            DNS_SEED: String::new(),
            TESTNET_PORT: String::new(),
            TESTNET_START_STRING: [0;4],
            PROTOCOL_VERSION: 0,
            USER_AGENT: String::new()
        };

        for line in reader.lines() {
            let current_line = line?;
            let setting: Vec<&str> = current_line.split('=').collect();

            if setting.len() != 2 {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid config input: {}", current_line),
                )));
            }
            Self::load_setting(&mut cfg, setting[0], setting[1])?;
        }
        Ok(cfg)
    }

    fn load_setting(&mut self, name: &str, value: &str) -> Result<(), Box<dyn Error>> {
        match name {
            "NUMBER_OF_NODES" => self.NUMBER_OF_NODES = usize::from_str(value)?,
            "DNS_SEED" => self.DNS_SEED = String::from(value),
            "TESTNET_PORT" => self.TESTNET_PORT = String::from(value),
            "TESTNET_START_STRING" => {
                self.TESTNET_START_STRING = i32::from_str(value)?.to_le_bytes();
            }
            "PROTOCOL_VERSION" => self.PROTOCOL_VERSION = i32::from_str(value)?,
            "USER_AGENT" => self.USER_AGENT = String::from(value),
            _ => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid config setting name: {}", name),
                )))
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_con_formato_invalido() {
        // GIVEN: un reader con contenido invalido para el archivo de configuracion
        let content = "Hola Mundo!".as_bytes();

        // WHEN: se ejecuta la funcion from_reader con ese reader
        let cfg = Config::from_reader(content);

        // THEN: la funcion devuelve un Err porque el contenido es invalido
        assert!(cfg.is_err());
        assert!(matches!(cfg, Err(_)));
    }

    #[test]
    fn config_sin_valores_requeridos() -> Result<(), Box<dyn Error>> {
        // GIVEN: un reader con contenido de configuracion completo
        let content = "NUMBER_OF_NODES=8\n\
        DNS_SEED=prueba\n\
        TESTNET_PORT=65536\n
        TESTNET_START_STRING=123456\n
        PROTOCOL_VERSION=70015\n
        USER_AGENT=/satoshi/"
            .as_bytes();

        // WHEN: se ejecuta la funcion from_reader con ese reader
        let cfg = Config::from_reader(content)?;

        // THEN: la funcion devuelve Ok y los parametros de configuracion tienen los valores esperados
        assert_eq!(8, cfg.NUMBER_OF_NODES);
        assert_eq!("prueba", cfg.DNS_SEED);
        assert_eq!("65536", cfg.TESTNET_PORT);
        Ok(())
    }

    #[test]
    fn config_con_argumento_faltante() {
        // GIVEN: un argumento sin file_path
        let args = [String::from("Bitcoin")];

        // WHEN: se ejecuta la funcion from con ese argumento
        let cfg = Config::from(&args);

        // THEN: la funcion devuelve un Err porque el contenido es invalido
        assert!(cfg.is_err());
        assert!(matches!(cfg, Err(_)));
    }

    #[test]
    fn config_con_argumentos_sobrantes() {
        // GIVEN: un array con 3 argumentos
        let args = [
            String::from("Bitcoin"),
            String::from("/path/nodo.conf"),
            String::from("arg_extra"),
        ];

        // WHEN: se ejecuta la funcion from con ese argumento
        let cfg = Config::from(&args);

        // THEN: la funcion devuelve un Err porque el contenido es invalido
        assert!(cfg.is_err());
        assert!(matches!(cfg, Err(_)));
    }
}
