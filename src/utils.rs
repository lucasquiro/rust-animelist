use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::path::Path;
use std::process::Command;
use chrono::NaiveDate;
use std::fmt::Display;

// Aquí abajo continúan tus funciones: ruta(), solicitar_numero(), etc.

// Esta funcion se encarga de recibir la ruta del archivo de entrada desde el usuario y devuelve un pathbuf con la ruta ingresada
pub fn ruta() -> PathBuf{
    
    // Se crea una variable mutable y vacia
    let mut input_menu = String::new();

    // Se recibe el valor ingresado por el usuario y se guarda en la variable input_menu
    io::stdin().read_line( &mut input_menu).expect("No se pudo leer el input del menu");

    /*se sanetiza el valor eliminando espacios al inicio las \ y el \n del final*/
    let menu_eleccion = input_menu.trim().trim_end().trim_matches('"').replace('\\', "");

    std::path::PathBuf::from(menu_eleccion)
}

pub fn obtener_fecha_hoy() -> String {
    let output = Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .expect("No se pudo obtener la fecha del sistema");
    
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn validar_formato_fecha(fecha_str: &str) -> Result<NaiveDate, String> {
    // NaiveDate::parse_from_str intenta encajar el texto en el formato "%Y-%m-%d"
    // %Y = Año de 4 dígitos, %m = Mes de 2 dígitos, %d = Día de 2 dígitos
    match NaiveDate::parse_from_str(fecha_str, "%Y-%m-%d") {
        Ok(fecha_valida) => Ok(fecha_valida),
        Err(_) => Err("El formato es incorrecto o la fecha no existe (ej: 32 de febrero).".to_string()),
    }
}

pub fn limpiar_consola<T: Display>(mensaje: T) {
print!("\x1B[2J\x1B[H");
io::stdout().flush().unwrap();

    println!("{}", mensaje);
}

pub fn solicitar_entrada(pregunta: &str) -> String {
    // Imprimimos la pregunta
    print!("{}", pregunta);
    // Forzamos a que se muestre en consola inmediatamente (ya que print! no hace salto de línea)
    io::stdout().flush().expect("No se pudo vaciar el búfer de salida");

    let mut respuesta = String::new();
    // Leemos la línea que ingresa el usuario
    io::stdin()
        .read_line(&mut respuesta)
        .expect("Error al leer la línea desde la consola");

    // .trim() quita espacios vacíos y el salto de línea (\n) del final de la respuesta
    respuesta.trim().to_string()
}

pub fn boveda_checker<P: AsRef<Path>>(boveda_path: P) -> bool {
    let obsidian_dir = boveda_path.as_ref().join(".obsidian");

    obsidian_dir.exists() && obsidian_dir.is_dir()
}


pub fn carpeta_esta_vacia<P: AsRef<Path>>(ruta: P) -> bool {
    let ruta = ruta.as_ref();

    // Si no existe o no es un directorio, la consideramos "no vacía"
    if !ruta.is_dir() {
        return false;
    }

    match fs::read_dir(ruta) {
        Ok(mut entradas) => entradas.next().is_none(),
        Err(_) => false,
    }
}

pub fn regenerar_md() {

}

pub fn mover_imagenes(ruta_antigua: &Path, ruta_nueva: &Path) -> io::Result<()> {
    if !ruta_antigua.exists() {
        return Ok(());
    }

    fs::create_dir_all(ruta_nueva)?;

    let mut entradas = fs::read_dir(ruta_antigua)?;

    if entradas.next().is_none() {
        println!("No hay archivos para mover.");
        return Ok(());
    }

    // Volvemos a leer el directorio porque consumimos el iterador
    for entrada in fs::read_dir(ruta_antigua)? {
        let entrada = entrada?;
        let origen = entrada.path();
        let destino = ruta_nueva.join(entrada.file_name());

        fs::rename(origen, destino)?;
    }

    Ok(())
}

pub fn solicitar_numero(menu: &str, minimo: u32, maximo: u32) -> u32 {
    loop {
        // Imprimimos todo el menú descriptivo (puede tener saltos de línea)
        println!("{}", menu);
        
        // Dejamos el indicador en la misma línea usando print!
        print!("Selecciona una opción (entre {} y {}): ", minimo, maximo);
        io::stdout().flush().expect("No se pudo vaciar el búfer de salida");

        let mut entrada = String::new();
        io::stdin()
            .read_line(&mut entrada)
            .expect("Error al leer la consola");

        // Intentamos transformar el texto a número entero limpio
        match entrada.trim().parse::<u32>() {
            Ok(num) => {
                // Verificamos si el número está dentro del rango válido de la pregunta
                if num >= minimo && num <= maximo {
                    return num; // Corta el loop y devuelve el número correcto
                } else {
                    println!("Error: El número debe estar entre {} y {}.\n", minimo, maximo);
                }
            }
            Err(_) => {
                println!("Error: Por favor, introduce un número válido.\n");
            }
        }
    }
}