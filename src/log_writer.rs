use std::{fs::{File, OpenOptions}, io::Write, error::Error, fmt, sync::mpsc::{Sender, channel, Receiver}, thread::{JoinHandle, self}};
use chrono::{Local, Timelike, Datelike};



#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LoggingError {
    WritingInFileError(String),
    ClosingFileError(String),
    OpeningFileError(String),
    ThreadJoinError(String),
}

impl fmt::Display for LoggingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoggingError::ClosingFileError(msg) => write!(f, "Error trying to close the log file: {}", msg),
            LoggingError::OpeningFileError(msg) => write!(f, "Error trying to open/create the log file: {}", msg),
            LoggingError::WritingInFileError(msg) => write!(f, "Error trying to write the log file: {}", msg),
            LoggingError::ThreadJoinError(msg) => write!(f, "Error trying to join the log thread: {}", msg),
        }
    }
}

impl Error for LoggingError {}

pub struct LogWriter {
    log_file: String,
}




impl LogWriter{
     
    pub fn new(log_file: String) -> Self {
        LogWriter { log_file }
    }

    pub fn create_logger(&self) -> Result<(Sender<String>, JoinHandle<()>), LoggingError> {
        let (tx, rx): (Sender<String>, Receiver<String>) = channel();
        let mut file = OpenOptions::new().create(true).append(true).open(&self.log_file).map_err(|err| LoggingError::OpeningFileError(err.to_string()))?;
        let local = Local::now();
        let date = format!("-------------------Actual date: {}-{}-{} Hour: {}:{}-------------------", local.day(), local.month(), local.year(), local.hour(), local.minute());    
        if let Err(err) = writeln!(file, "{}", date) {
            println!("ERROR AL ESCRIBIR LA FECHA DE LOGGING: {}, {}", date, LoggingError::WritingInFileError(err.to_string()));
        }
        let handle = thread::spawn(move || {
            for log in rx {
                if let Err(err) = writeln!(file, "{}", log) {
                    println!("Error {} al escribir en el log: {}", LoggingError::WritingInFileError(err.to_string()), log);
                };
            }
        });
        Ok((tx, handle))
    }
    


}


/* 
#[cfg(test)]
mod test{
    use std::{io::{BufReader, BufRead}, fs::File};

    use super::LogWriter;


    #[test]
    fn test_el_archivo_se_crea_correctamente(){
        let log_writer: Result<LogWriter, std::io::Error>=LogWriter::create_log_writer("src/log.txt");
        assert!(log_writer.is_ok());
    }
    #[test]
    fn test_se_escribe_correctamentamente_en_el_archivo()->Result<(),std::io::Error>{
        let mut log_writer: LogWriter=LogWriter::create_log_writer("src/log.txt")?;
        let contenido_escrito = "Hola Mundo";
        log_writer.write(contenido_escrito)?;
        let file = File::open("src/log.txt")?;
        let lector = BufReader::new(file);
        let mut lineas = Vec::new();

        for linea_resultado in lector.lines() {
            let linea = linea_resultado?.to_string();
            lineas.push(linea);
        }
        let contenido_esperado=lineas[0].as_str();
        assert_eq!(contenido_escrito,contenido_esperado);
        Ok(())
    }
}

*/