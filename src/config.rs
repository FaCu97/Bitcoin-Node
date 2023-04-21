use std::fs;

pub struct Config {
    pub file_path: String,
    pub var1: String,
    pub var2: String,
    pub var3: String,
}
impl Config {
    fn load(file_path: String) -> Result<Config, &'static str> {
        // me da error si saco el .expect y manejo el error con if let, por ahora lo dejo así.
        let content =
            fs::read_to_string(&file_path).expect("Should have been able to read the file");
        //    if let Err(e) = content {
        //        return Err("Error al leer el archivo de configuracion");
        //    }

        // Acá hay que iterar y crear las variables
        let contenido_archivo = content;
        let mut valores = Vec::new();
        for linea in contenido_archivo.lines() {
            let partes = linea.split("=");
            let valor = partes.collect::<Vec<&str>>()[1];
            valores.push(valor);
        }

        Ok(Config {
            file_path: file_path,
            var1: String::from(valores[0]),
            var2: String::from(valores[1]),
            var3: String::from(valores[2]),
        })
    }
    pub fn from(args: &[String]) -> Result<Config, &'static str> {
        if args.len() > 2 {
            return Err("Too many arguments");
        }

        if args.len() < 2 {
            return Err("Not enough arguments");
        }

        let file_path = args[1].clone();
        let config = Self::load(file_path);
        config
    }
}
