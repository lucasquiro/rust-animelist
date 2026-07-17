// 1. Esto le dice a Rust: "Busca un archivo llamado config.rs en esta misma carpeta y trátalo como un módulo"
mod config;
mod utils;


// Traemos cargar_config al alcance[cite: 1]
use config::{verificar_config, cargar_config, Config};

use crate::utils::solicitar_numero;

// 2. Traemos al alcance local solo las funciones que realmente vas a usar en main

const OPCIONES_VALIDAS: [&str; 2] = ["ANIMES", "Editar configuración"];

fn main() {

    let _config_data: Config;

    // Primero verificamos si el archivo de configuración existe en caso de que si, cargamos las variables en memoria, y si no lo creamos

    if verificar_config() {
        println!("El archivo de configuración existe. Cargando...");
        
        // Cargamos el JSON directamente a la variable en memoria
        _config_data = cargar_config().expect("Error crítico: El archivo config.json está corrupto o no es válido");
    } else {
        // Crear_config() crea el archivo y ruta de configuracion con un archivo vacio
        config::crear_config();

        // Ahora llamamos a cada pregunta de forma individual (en lugar de preguntas_config())
        // para poder reutilizarlas más adelante en la edición de configuración.
        let boveda_path = config::pregunta_boveda_path();
        let (final_absolute_path, final_relative_path) = config::pregunta_imagenes_path(&boveda_path);
        let cover_size = config::pregunta_cover_size();
        let (sistema_calificacion, usar_emojis) = config::pregunta_sistema_calificacion();
        let preferencia_nombre = config::pregunta_preferencia_nombre();
        let fecha_corte_historico = config::pregunta_fecha_corte();
        let config_path = config::obtener_ruta_carpeta_base();

        _config_data = Config {
            boveda_path,
            final_absolute_path,
            final_relative_path,
            config_path,
            cover_size,
            sistema_calificacion,
            usar_emojis,
            preferencia_nombre,
            fecha_corte_historico,
        };

        // Necesitamos la ruta donde vive el config.json para pasársela a rellenar_config()
        if let Some(ruta_guardado) = config::obtener_ruta_config() {
            // Una ves que se hicieron las preguntas, se guarda el config.json
            match config::rellenar_config(&_config_data, &ruta_guardado) {
                Ok(()) => println!("Configuración guardada correctamente."),
                Err(e) => eprintln!("Error al guardar la configuración: {}", e),
            }
        } else {
            eprintln!("Error crítico: No se pudo determinar el directorio HOME de tu sistema Linux.");
        }
    }
        
    // 1. Creamos el encabezado del menú
    let mut menu = String::from("\n── Menú de opciones ──\nSeleccione la opción que prefiere:\n\n");

        // 2. Iteramos dinámicamente sobre la constante usando .iter() y .enumerate()
        // 'i' es el índice (0, 1, 2...) y 'opcion' es el texto ("ANIMES", "Editar config"...)
        for (i, opcion) in OPCIONES_VALIDAS.iter().enumerate() {
            // Vamos añadiendo cada línea formateada al String usando la macro writeln!
            use std::fmt::Write; // Necesario para poder usar writeln! sobre un String
            let _ = writeln!(menu, "    {}- {}", i + 1, opcion);
        }

    // 3. Ahora 'menu' contiene todo el texto armado listo para imprimirse o usarse
    utils::limpiar_consola(menu);

    loop {
        let opciones_num = OPCIONES_VALIDAS.len() as u32;
        let opciones = solicitar_numero("", 1, opciones_num);

        match opciones {
            1 => {
                anime();
                break;
            }
            2 => {
                config::editar_config();
                break;
            }
            _ => {
                print!("Lo que ingresaste no es una opcion valida.");
                continue;
            }
        }
    }
}

fn anime() {

}