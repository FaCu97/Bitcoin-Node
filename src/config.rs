use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::str::FromStr;
pub struct Config {
    pub var1: u16,
    pub var2: String,
    pub var3: u32,
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
            var1: 0,
            var2: String::new(),
            var3: 0,
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
            "CONFIG_EJEMPLO1" => self.var1 = u16::from_str(value)?,
            "CONFIG_EJEMPLO2" => self.var2 = String::from(value),
            "CONFIG_EJEMPLO3" => self.var3 = u32::from_str(value)?,
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
        let content = "CONFIG_EJEMPLO2=prueba\n\
        CONFIG_EJEMPLO1=9876\n\
        CONFIG_EJEMPLO3=65536"
            .as_bytes();

        // WHEN: se ejecuta la funcion from_reader con ese reader
        let cfg = Config::from_reader(content)?;

        // THEN: la funcion devuelve Ok y los parametros de configuracion tienen los valores esperados
        assert_eq!(9876, cfg.var1);
        assert_eq!("prueba", cfg.var2);
        assert_eq!(65536, cfg.var3);
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
