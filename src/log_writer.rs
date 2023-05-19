use std::{
    fs::{File, OpenOptions},
    io::Write,
};

pub struct LogWriter {
    log_file: File,
}

impl LogWriter {
    fn new(log_file: File) -> Self {
        LogWriter { log_file }
    }
    /// crea una nueva estructura con el archivo correspondiente en caso de que el
    /// archivo no exista este se crea , caso contrario se abre el mismo sin pisar
    /// el contenido del mismo
    pub fn create_log_writer(path: &str) -> Result<LogWriter, std::io::Error> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self::new(file))
    }
    /// Funcion para escribir en el archivo dentro del struct
    pub fn write(&mut self, mssg: &str) -> Result<(), std::io::Error> {
        writeln!(self.log_file, "{}", mssg)
    }
}

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
