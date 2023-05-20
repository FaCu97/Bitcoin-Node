use chrono::{Datelike, Local, Timelike};
use std::{
    error::Error,
    fmt,
    fs::{File, OpenOptions},
    io::Write,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
};

const CENTER_DATE_LINE: &str = "-------------------------------------------";

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
            LoggingError::ClosingFileError(msg) => {
                write!(f, "Error trying to close the log file: {}", msg)
            }
            LoggingError::OpeningFileError(msg) => {
                write!(f, "Error trying to open/create the log file: {}", msg)
            }
            LoggingError::WritingInFileError(msg) => {
                write!(
                    f,
                    "Error trying to write the log file or send through log thread channel: {}",
                    msg
                )
            }
            LoggingError::ThreadJoinError(msg) => {
                write!(f, "Error trying to join the log thread: {}", msg)
            }
        }
    }
}

impl Error for LoggingError {}

type LogFileSender = Sender<String>;
type Loggers = (
    LogFileSender,
    JoinHandle<()>,
    LogFileSender,
    JoinHandle<()>,
    LogFileSender,
    JoinHandle<()>,
);

pub fn write_in_log(log_sender: LogFileSender, msg: &str) {
    if let Err(err) = log_sender.send(msg.to_string()) {
        println!(
            "Error al intentar escribir {} en el log!, error: {}\n",
            msg, err
        );
    };
}

pub fn set_up_loggers(
    error_file_path: String,
    info_file_path: String,
    message_file_path: String,
) -> Result<Loggers, LoggingError> {
    let (error_log_sender, error_handler) = LogWriter::new(error_file_path).create_logger()?;
    let (info_log_sender, info_handler) = LogWriter::new(info_file_path).create_logger()?;
    let (message_log_sender, message_handler) =
        LogWriter::new(message_file_path).create_logger()?;
    Ok((
        error_log_sender,
        error_handler,
        info_log_sender,
        info_handler,
        message_log_sender,
        message_handler,
    ))
}

pub fn shutdown_loggers(
    log_sender: LogSender,
    error_handler: JoinHandle<()>,
    info_handler: JoinHandle<()>,
    message_handler: JoinHandle<()>,
) -> Result<(), LoggingError> {
    shutdown_logger(log_sender.info_log_sender, info_handler)?;
    shutdown_logger(log_sender.error_log_sender, error_handler)?;
    shutdown_logger(log_sender.messege_log_sender, message_handler)?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct LogSender {
    pub error_log_sender: LogFileSender,
    pub info_log_sender: LogFileSender,
    pub messege_log_sender: LogFileSender,
}

impl LogSender {
    pub fn new(
        error_log_sender: LogFileSender,
        info_log_sender: LogFileSender,
        messege_log_sender: LogFileSender,
    ) -> Self {
        LogSender {
            error_log_sender,
            info_log_sender,
            messege_log_sender,
        }
    }
}
/// Representa a la estructura que escribe en el archivo Log
pub struct LogWriter {
    log_file: String,
}

impl LogWriter {
    /// Dado un string con el path al archivo, se encarga de crear un LogWriter y gurada el nombre del archivo
    /// en el campo log_file
    pub fn new(log_file: String) -> Self {
        LogWriter { log_file }
    }

    /// Recibe un LogWriter y se encarga de abrir/crear el archivo del logWiriter y crear un thread que va a estar constantemente escuchando por el
    /// channel logs para escribir en el archivo log. Escribe la fecha actual apenas abre el archivo. En caso de que haya un error
    /// lo imprime por consola y sigue escuchando. Devuelve el extremo para mandar por el channel y el JoinHandle del thread en una tupla.
    /// #use_example
    /// let log = LogWriter::new("archivo_log.txt");
    /// let (tx, handle) = log.create_logger()?;
    /// tx.send("first log!!".to_string())?;
    /// calling_function(tx.clone(), ...);
    /// tx.send("second log!!".to_string())?
    /// log.shutdown(tx, handle)?;
    pub fn create_logger(&self) -> Result<(LogFileSender, JoinHandle<()>), LoggingError> {
        let (tx, rx): (Sender<String>, Receiver<String>) = channel();
        let mut file = open_log_file(&self.log_file)?;
        let local = Local::now();
        let date = format!(
            "\n{} Actual date: {}-{}-{} Hour: {:02}:{:02}:{:02} {}\n",
            CENTER_DATE_LINE,
            local.day(),
            local.month(),
            local.year(),
            local.hour(),
            local.minute(),
            local.second(),
            CENTER_DATE_LINE
        );
        if let Err(err) = writeln!(file, "{}", date) {
            println!(
                "Error al escribir la fecha de logging: {}, {}",
                date,
                LoggingError::WritingInFileError(err.to_string())
            );
        }
        let handle = thread::spawn(move || {
            for log in rx {
                let date = format!(
                    "{}:{}:{:02}",
                    Local::now().hour(),
                    Local::now().minute(),
                    Local::now().second()
                );
                if let Err(err) = writeln!(file, "{}: {}", date, log) {
                    println!(
                        "Error {} al escribir en el log: {}",
                        LoggingError::WritingInFileError(err.to_string()),
                        log
                    );
                };
            }
        });
        Ok((tx, handle))
    }
}

fn open_log_file(log_file: &String) -> Result<File, LoggingError> {
    let log_open_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .map_err(|err| LoggingError::OpeningFileError(err.to_string()))?;
    Ok(log_open_file)
}
/// Dado el extremo para escribir por el channel y un JoinHandle del thread que esta escribiendo en el archivo log,
/// imprime que va a cerrar el archivo, cierra el extremo del channel y le hace join al thread para que termine. Devuelve
/// error en caso de que no se pueda mandar el mensaje por el channel o no se pueda hacer join correctamente al thread
fn shutdown_logger(tx: LogFileSender, handler: JoinHandle<()>) -> Result<(), LoggingError> {
    tx.send("Closing log\n".to_string())
        .map_err(|err| LoggingError::WritingInFileError(err.to_string()))?;
    drop(tx);
    handler
        .join()
        .map_err(|err| LoggingError::ThreadJoinError(format!("{:?}", err)))?;
    Ok(())
}

/*
#[cfg(test)]
mod test {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use super::LogWriter;

    #[test]
    fn test_el_archivo_se_crea_correctamente() {
        let log_writer: Result<LogWriter, std::io::Error> =
            LogWriter::create_log_writer("src/log.txt");
        assert!(log_writer.is_ok());
    }
    #[test]
    fn test_se_escribe_correctamentamente_en_el_archivo() -> Result<(), std::io::Error> {
        let mut log_writer: LogWriter = LogWriter::create_log_writer("src/log.txt")?;
        let contenido_escrito = "Hola Mundo";
        log_writer.write(contenido_escrito)?;
        let file = File::open("src/log.txt")?;
        let lector = BufReader::new(file);
        let mut lineas = Vec::new();

        for linea_resultado in lector.lines() {
            let linea = linea_resultado?.to_string();
            lineas.push(linea);
        }
        let contenido_esperado = lineas[0].as_str();
        assert_eq!(contenido_escrito, contenido_esperado);
        Ok(())
    }
}
 */
