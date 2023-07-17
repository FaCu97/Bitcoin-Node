use std::{sync::{Arc, RwLock}, net::TcpStream, error::Error, path::Path};

use crate::{config::Config, logwriter::log_writer::{LogSender, write_in_log}, blocks::block_header::BlockHeader};

use super::{read_first_headers_from_disk, DownloadError, download_first_headers};

pub fn get_initial_headers(
    config: &Arc<Config>,
    log_sender: &LogSender,
    headers: Arc<RwLock<Vec<BlockHeader>>>,
    nodes: Arc<RwLock<Vec<TcpStream>>>,
) -> Result<(), Box<dyn Error>> {
    if Path::new(&config.archivo_headers).exists() {
        read_first_headers_from_disk(config, log_sender, headers).map_err(|err| {
            write_in_log(
                &log_sender.error_log_sender,
                format!("Error al descargar primeros 2 millones de headers de disco. {err}")
                    .as_str(),
            );
            DownloadError::CanNotRead(format!(
                "Error al leer primeros 2 millones de headers. {}",
                err
            ))
        })?;
    } else {
        download_first_headers(config, log_sender, headers, nodes)?;
    }
    Ok(())
}