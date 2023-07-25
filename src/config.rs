use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::str::FromStr;
use std::sync::Arc;

use crate::custom_errors::NodeCustomErrors;

/// Permite validar la cantidad de atributos en el archivo de configuración
/// Si se agregan hay que incrementarlo
const CANTIDAD_ATRIBUTOS: usize = 23;

/// Almacena los campos leidos del archivo de configuración
#[derive(Debug, Clone)]
pub struct Config {
    pub number_of_nodes: usize,
    pub dns_seed: String,
    pub connect_to_dns_nodes: bool,
    pub custom_nodes_ips: Vec<String>,
    pub net_port: u16,
    pub start_string: [u8; 4],
    pub protocol_version: i32,
    pub user_agent: String,
    pub n_threads: usize,
    pub connect_timeout: u64,
    pub max_connections_to_server: u8,
    pub error_log_path: String,
    pub info_log_path: String,
    pub message_log_path: String,
    pub blocks_download_per_node: usize,
    pub fecha_inicio_proyecto: String,
    pub formato_fecha_inicio_proyecto: String,
    pub headers_in_disk: usize,
    pub read_headers_from_disk: bool,
    pub ibd_single_node: bool,
    pub height_first_block_to_download: usize,
    pub archivo_headers: String,
    pub logs_folder_path: String,
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
    pub fn from(args: &[String]) -> Result<Arc<Self>, NodeCustomErrors> {
        if args.len() > 2 {
            return Err(NodeCustomErrors::ArgumentsError(
                "Too many arguments".to_string(),
            ));
        }

        if args.len() < 2 {
            return Err(NodeCustomErrors::ArgumentsError(
                "Not enough arguments".to_string(),
            ));
        }
        let file = File::open(&args[1])
            .map_err(|err| NodeCustomErrors::OpeningFileError(err.to_string()))?;
        Self::from_reader(file).map_err(|err| NodeCustomErrors::ReadingFileError(err.to_string()))
    }

    /// Lee del file recibido y devuelve el struct de configuración inicializado.
    fn from_reader<T: Read>(content: T) -> Result<Arc<Config>, Box<dyn Error>> {
        let reader = BufReader::new(content);

        let mut cfg = Self {
            number_of_nodes: 0,
            dns_seed: String::new(),
            connect_to_dns_nodes: true,
            custom_nodes_ips: Vec::new(),
            net_port: 0,
            start_string: [0; 4],
            protocol_version: 0,
            user_agent: String::new(),
            n_threads: 0,
            connect_timeout: 0,
            max_connections_to_server: 0,
            error_log_path: String::new(),
            info_log_path: String::new(),
            message_log_path: String::new(),
            blocks_download_per_node: 0,
            fecha_inicio_proyecto: String::new(),
            formato_fecha_inicio_proyecto: String::new(),
            headers_in_disk: 0,
            read_headers_from_disk: false,
            ibd_single_node: false,
            height_first_block_to_download: 0,
            archivo_headers: String::new(),
            logs_folder_path: String::new(),
        };

        let mut number_of_settings_loaded: usize = 0;
        for line in reader.lines() {
            let current_line = line?;
            // es un comentario, ignorarlo
            if current_line.starts_with('#') {
                continue;
            }
            let setting: Vec<&str> = current_line.split('=').collect();

            if setting.len() != 2 {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid config input: {}", current_line),
                )));
            }
            Self::load_setting(
                &mut cfg,
                setting[0],
                setting[1],
                &mut number_of_settings_loaded,
            )?;
        }
        Self::check_number_of_attributes(number_of_settings_loaded)?;
        Ok(Arc::new(cfg))
    }

    /// Chequea la cantidad atributos contra la cantidad leida.
    /// Devuelve error en caso de haber diferencia
    fn check_number_of_attributes(cantidad_de_lineas: usize) -> Result<(), Box<dyn Error>> {
        if cantidad_de_lineas != CANTIDAD_ATRIBUTOS {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid quantity of lines in file config".to_string(),
            )));
        }
        Ok(())
    }

    /// Recibe el nombre del atributo y lo guarda en el struct de configuración.
    /// Actualiza la cantidad de atributos leidos para su posterior verificación.
    fn load_setting(
        &mut self,
        name: &str,
        value: &str,
        number_of_settings_loaded: &mut usize,
    ) -> Result<(), Box<dyn Error>> {
        match name {
            "NUMBER_OF_NODES" => {
                self.number_of_nodes = usize::from_str(value)?;
                *number_of_settings_loaded += 1;
            }
            "DNS_SEED" => {
                self.dns_seed = String::from(value);
                *number_of_settings_loaded += 1;
            }
            "CONNECT_TO_DNS_NODES" => {
                self.connect_to_dns_nodes = bool::from_str(value)?;
                *number_of_settings_loaded += 1;
            }
            "CUSTOM_NODES_IPS" => {
                if !value.is_empty() {
                    self.custom_nodes_ips = value.split(',').map(String::from).collect();
                }
                *number_of_settings_loaded += 1;
            }
            "NET_PORT" => {
                self.net_port = u16::from_str(value)?;
                *number_of_settings_loaded += 1;
            }
            "START_STRING" => {
                self.start_string = i32::from_str(value)?.to_be_bytes();
                *number_of_settings_loaded += 1;
            }
            "PROTOCOL_VERSION" => {
                self.protocol_version = i32::from_str(value)?;
                *number_of_settings_loaded += 1;
            }
            "USER_AGENT" => {
                self.user_agent = String::from(value);
                *number_of_settings_loaded += 1;
            }
            "N_THREADS" => {
                self.n_threads = usize::from_str(value)?;
                *number_of_settings_loaded += 1;
            }
            "CONNECT_TIMEOUT" => {
                self.connect_timeout = u64::from_str(value)?;
                *number_of_settings_loaded += 1;
            }
            "MAX_CONNECTIONS" => {
                self.max_connections_to_server = u8::from_str(value)?;
                *number_of_settings_loaded += 1;
            }
            "ERROR_LOG_PATH" => {
                self.error_log_path = String::from(value);
                *number_of_settings_loaded += 1;
            }
            "INFO_LOG_PATH" => {
                self.info_log_path = String::from(value);
                *number_of_settings_loaded += 1;
            }
            "MESSAGE_LOG_PATH" => {
                self.message_log_path = String::from(value);
                *number_of_settings_loaded += 1;
            }
            "BLOCKS_DOWNLOAD_PER_NODE" => {
                self.blocks_download_per_node = usize::from_str(value)?;
                *number_of_settings_loaded += 1;
            }
            "FECHA_INICIO_PROYECTO" => {
                self.fecha_inicio_proyecto = String::from(value);
                *number_of_settings_loaded += 1;
            }
            "FORMATO_FECHA_INICIO_PROYECTO" => {
                self.formato_fecha_inicio_proyecto = String::from(value);
                *number_of_settings_loaded += 1;
            }
            "AMOUNT_OF_HEADERS_TO_STORE_IN_DISK" => {
                self.headers_in_disk = usize::from_str(value)?;
                *number_of_settings_loaded += 1;
            }
            "READ_HEADERS_FROM_DISK" => {
                self.read_headers_from_disk = bool::from_str(value)?;
                *number_of_settings_loaded += 1;
            }
            "DOWNLOAD_FULL_BLOCKCHAIN_FROM_SINGLE_NODE" => {
                self.ibd_single_node = bool::from_str(value)?;
                *number_of_settings_loaded += 1;
            }
            "HEIGHT_FIRST_BLOCK_TO_DOWNLOAD" => {
                self.height_first_block_to_download = usize::from_str(value)?;
                *number_of_settings_loaded += 1;
            }
            "ARCHIVO_HEADERS" => {
                self.archivo_headers = String::from(value);
                *number_of_settings_loaded += 1;
            }
            "CARPETA_LOGS" => {
                self.logs_folder_path = String::from(value);
                *number_of_settings_loaded += 1;
            }
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
    fn config_file_completo_se_crea_correctamente() -> Result<(), Box<dyn Error>> {
        // GIVEN: un archivo de configuracion completo
        let file = File::open("nodo.conf")?;

        // WHEN: se ejecuta la funcion from_reader con ese archivo
        let cfg_result = Config::from_reader(file);

        // THEN: se crea correctamente sin dar error
        assert!(!cfg_result.is_err());
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

    #[test]
    fn archivo_config_con_cantidad_atributos_incorrecta_devuelve_error(
    ) -> Result<(), Box<dyn Error>> {
        // GIVEN: Un archivo de configuración con cantidad incorrecta de lineas
        let content = "NUMBER_OF_NODES=8\n\
        DNS_SEED=prueba\n\
        TESTNET_PORT=65536\n\
        TESTNET_START_STRING=123456\n\
        PROTOCOL_VERSION=70015\n\
        USER_AGENT=/satoshi/"
            .as_bytes();

        // WHEN: se crea la la configuración
        let config_result = Config::from_reader(content);

        // THEN: la configuración da error
        assert!(config_result.is_err());
        Ok(())
    }
}
