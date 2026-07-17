use std::fs;
use std::path::PathBuf;
use std::path::Path;
use serde::{Serialize, Deserialize};
use std::io::{Write, self, BufReader};
use serde_json;
use std::fs::File;
use chrono::NaiveDate;
use crate::utils;
use crate::utils::limpiar_consola;
use crate::utils::obtener_fecha_hoy;
use std::fs::remove_dir_all;

pub fn crear_config() {
    // En lugar de unwrap, usamos "if let" para actuar solo si la ruta es válida
    if let Some(ruta_config) = obtener_ruta_config() {
            
        // 1. Crear los directorios padres de forma segura (.parent() nos da la carpeta)
        if let Some(carpeta_padre) = ruta_config.parent() {
            std::fs::create_dir_all(carpeta_padre)
                .expect("No se pudo crear la carpeta en ~/.config/Anime-Obsidean");
        }

        // 2. Aquí es donde harás las preguntas al usuario (Clap / standard input)
        // Por ahora, solo creamos el archivo vacío para asegurar que funciona:
        std::fs::File::create(&ruta_config)
            .expect("No se pudo crear el archivo config.json");
            
    } else {
        // Si obtener_ruta_config() devolvió None, detenemos el programa con un mensaje claro
        panic!("Error crítico: No se pudo determinar el directorio HOME de tu sistema Linux.");
    }
}

/// Pregunta 1: solicita y valida la ruta de la bóveda de Obsidian.
pub fn pregunta_boveda_path() -> PathBuf {
    let menu = String::from("\n── Ruta de la bóveda de Obsidian ──\n\nIngresa la ruta completa de tu bóveda de Obsidian.\nEjemplo: /home/usuario/Documentos/ObsidianAnime\n");
    println!("{}", menu);

    let mut boveda_path: PathBuf;

    loop {
        boveda_path = utils::ruta();

        // 1. Primera comprobación: ¿Existe la ruta?
        if !boveda_path.exists() {
            println!("La ruta ingresada no existe. Por favor, verifícala.");
            continue; // Vuelve al inicio del loop a pedir la ruta otra vez
        }

        // 2. Segunda comprobación: ¿Es una bóveda de Obsidian?
        if !utils::boveda_checker(&boveda_path) {
            println!("La ruta existe, pero no es una bóveda de Obsidian válida.");
            println!("Por favor, crea la bóveda vacía en Obsidian y vuelve a ingresar la ruta.");
            continue; // Vuelve al inicio del loop a pedir la ruta otra vez
        }

        break;
    }

    boveda_path
}

/// Pregunta 2: solicita la ruta (absoluta o relativa) donde se guardarán las
/// imágenes de portada dentro de la bóveda. Devuelve la ruta absoluta real en
/// disco y la ruta relativa (con "/" inicial) que se guardará en el config.
pub fn pregunta_imagenes_path(boveda_path: &Path) -> (PathBuf, PathBuf) {
    let mut menu = String::from("\n── Ruta de las imágenes ──\n\n¿Deseas usar una ruta personalizada para las imágenes dentro de tu bóveda de Obsidian?\nEjemplo: /NombreBoveda/imagenes\n\n1. Sí, quiero usar una ruta personalizada\n2. No, prefiero usar la ruta predeterminada (/NombreBoveda/Anime/Recursos/Portadas)\n\nSe recomienda usar la ruta predeterminada\n");
    let opcion_ruta = utils::solicitar_numero(&menu, 1, 2);

    // Ruta predeterminada interna de Obsidian
    let default_relative = Path::new("Anime/Recursos/Portadas");
    let final_absolute_path;
    let final_relative_path;

    match opcion_ruta {
        1 => {
            // Entramos en un loop para permitir reintentar si la carpeta tiene archivos y el usuario dice NO
            loop {
                utils::limpiar_consola("\n── Asistente de configuración inicial ──");
                menu = String::from("\n── Ruta personalizada para las imágenes ──\n\nIngresa la ruta (puede ser absoluta o relativa a la bóveda):\n");
                println!("{}", menu);

                let entrada_usuario = utils::ruta();

                // 1. Normalizar la ruta absoluta y la relativa respecto a boveda_path
                let (abs_path, rel_path) = if entrada_usuario.is_absolute() {
                    // Si es absoluta, intentamos obtener la relativa respecto a la boveda
                    if let Ok(rel) = entrada_usuario.strip_prefix(boveda_path) {
                        (entrada_usuario.clone(), Path::new("/").join(rel))
                    } else {
                        // FIX: en Linux cualquier ruta que empieza con "/" se considera
                        // "absoluta", aunque el usuario en realidad quiso escribir una ruta
                        // relativa a la bóveda (ej: "/imagenes/portadas"). Si no coincide con
                        // ninguna ruta absoluta real dentro de la bóveda, la tratamos como
                        // relativa en lugar de rechazarla.
                        let rel_limpia = entrada_usuario.strip_prefix("/").unwrap_or(&entrada_usuario);
                        let abs = boveda_path.join(rel_limpia);
                        let rel = Path::new("/").join(rel_limpia);
                        (abs, rel)
                    }
                } else {
                    // Si es relativa (ej: "imagenes")
                    let rel_limpia = entrada_usuario.strip_prefix("/").unwrap_or(&entrada_usuario);
                    let abs = boveda_path.join(rel_limpia);
                    let rel = Path::new("/").join(rel_limpia);
                    (abs, rel)
                };

                // Rechazar la raíz de la bóveda como destino (evita esparcir portadas sueltas en el home de la bóveda)
                if rel_path == Path::new("/") {
                    println!("\nError: No puedes usar la raíz de la bóveda como ruta de imágenes.");
                    println!("Debes ingresar una ruta que esté al menos un nivel dentro de la bóveda (ej: imagenes, Recursos/Portadas, etc.).");
                    println!("Presiona Enter para intentar de nuevo...");
                    let _ = utils::solicitar_entrada("");
                    continue;
                }

                // 2. Validaciones sobre el directorio objetivo
                if abs_path.exists() {
                    if abs_path.is_dir() {
                        if utils::carpeta_esta_vacia(&abs_path) {
                            final_absolute_path = abs_path;
                            final_relative_path = rel_path;
                            break; // Ruta válida y vacía
                        } else {
                            // Preguntar si quiere usar la carpeta con archivos
                            let menu_pregunta = String::from("\nLa carpeta que ingresaste tiene archivos dentro. ¿Quieres usarla de todas formas?\n\n1. Sí\n2. No, quiero ingresar otra ruta\n");
                            let usar_carpeta_llena = utils::solicitar_numero(&menu_pregunta, 1, 2);

                            if usar_carpeta_llena == 1 {
                                final_absolute_path = abs_path;
                                final_relative_path = rel_path;
                                break; // El usuario acepta usarla
                            } else {
                                continue; // Reintentar loop desde el principio
                            }
                        }
                    } else {
                        println!("\nError: La ruta existe, pero no es un directorio válido (es un archivo).");
                        println!("Presiona Enter para intentar de nuevo...");
                        let _ = utils::solicitar_entrada("");
                        continue;
                    }
                } else {
                    // Si no existe, se intenta crear la estructura completa de directorios
                    if let Err(e) = fs::create_dir_all(&abs_path) {
                        println!("\nError al crear la carpeta: {}", e);
                        println!("Presiona Enter para intentar de nuevo...");
                        let _ = utils::solicitar_entrada("");
                        continue;
                    } else {
                        final_absolute_path = abs_path;
                        final_relative_path = rel_path;
                        break; // Creada con éxito
                    }
                }
            }
        }
        2 => {
            println!("\nHas elegido usar la ruta predeterminada para las imágenes.");
            final_absolute_path = boveda_path.join(default_relative);
            final_relative_path = Path::new("/").join(default_relative);

            // Crear la carpeta predeterminada si no existe
            if !final_absolute_path.exists() {
                let _ = fs::create_dir_all(&final_absolute_path);
            }
        }
        _ => unreachable!(), // utils::solicitar_numero ya garantiza que solo sea 1 o 2
    }

    (final_absolute_path, final_relative_path)
}

/// Pregunta 3: tamaño (en px) de las portadas de los animes.
pub fn pregunta_cover_size() -> u32 {
    let menu = String::from("\n── Tamaño de las portadas ──\n\nEste es un parámetro interno de la bóveda de Obsidian. Se recomienda 90.\n\nIngresa el tamaño de las portadas de los animes (entre 50 y 200):\n");
    utils::solicitar_numero(&menu, 50, 200)
}

/// Pregunta 4 (y 5): sistema de calificación elegido y, si corresponde,
/// si se quieren usar emojis para representar los gustos.
pub fn pregunta_sistema_calificacion() -> (u8, bool) {
    let mut menu = String::from("\n── Sistema de calificación ──\n\n1. Por gustos (No me gustó, Bueno, Me encantó, Favorito)\n2. Escala del 1 al 10\n3. Escala del 1 al 5\n4. Sin calificación\n");

    let tipo_calificacion = utils::solicitar_numero(&menu, 1, 4);

    // "as u8" es seguro acá porque solicitar_numero ya garantiza que tipo_calificacion está entre 1 y 4.
    let sistema_calificacion: u8 = tipo_calificacion as u8;
    let mut usar_emojis: bool = false;

    match tipo_calificacion {
        1 => {
            // En caso de que el usuario eligiera el sistema de calificación de gustos,
            // se le preguntará si quiere asignar emojis en los nombres representando los gustos
            menu = String::from("\n── Emojis para el sistema de gustos ──\n\n¿Quieres añadir emojis a los nombres de los animes para representar cuánto te gustaron?\n\n    No me gustó          ✖️\n    Neutral              (sin emoji)\n    Me encantó           ❤️\n    Es de mis favoritos  ⭐\n\n    Ejemplo: Bocchi the Rock ❤️.md\n\n1. Sí\n2. No\n");

            loop {
                let emojis = utils::solicitar_numero(&menu, 1, 2);

                if emojis == 1 {
                    usar_emojis = true;
                    break;
                } else if emojis == 2 {
                    usar_emojis = false;
                    break;
                } else {
                    menu = String::from("El número que ingresaste no corresponde a las opciones. Inténtalo de nuevo.");
                    continue;
                }
            }
        }
        2 => {}
        3 => {}
        4 => {}
        _ => {
            println!("Opción inválida. Por favor, selecciona una opción válida.");
        }
    }

    (sistema_calificacion, usar_emojis)
}

/// Pregunta 6: cómo prefiere el usuario que se guarden los nombres de los animes.
pub fn pregunta_preferencia_nombre() -> String {
    let menu = String::from("\n── Preferencia de nombre para los archivos ──\n\n1. Propio\n   Ej: Las Quintillizas\n\n2. Romaji japonés\n   Ej: Go-toubun no Hanayome\n\n3. Inglés\n   Ej: The Quintessential Quintuplets\n\n4. Nativo japonés\n   Ej: 五等分の花嫁\n");

    let preferencia_nombre = utils::solicitar_numero(&menu, 1, 4);

    // El struct Config guarda esto como String (para que sea legible directamente en el json),
    // así que traducimos la opción numérica elegida a su texto correspondiente.
    match preferencia_nombre {
        1 => "Propio".to_string(),
        2 => "Romaji Japones".to_string(),
        3 => "Ingles".to_string(),
        4 => "Nativo Japones".to_string(),
        _ => unreachable!(), // solicitar_numero ya garantiza que esté entre 1 y 4
    }
}

/// Pregunta 7: fecha de corte histórico para animes sin fecha registrada.
pub fn pregunta_fecha_corte() -> NaiveDate {
    let fecha_corte_historico: String;

    loop {
        utils::limpiar_consola("\n── Asistente de configuración inicial ──");
        let menu = String::from("\n── Fecha de corte histórico ──\n\n¿Recuerdas cuál fue la última fecha exacta en la que viste un anime?\n\nEsa fecha se usará para los animes que ingreses sin fecha registrada.\n");
        println!("{}", menu);
        let entrada_fecha = utils::solicitar_entrada("Fecha (AAAA-MM-DD) o ENTER para hoy: ");

        if entrada_fecha.is_empty() {
            // Si da ENTER, obtenemos la fecha de hoy usando chrono de forma nativa en memoria
            fecha_corte_historico = obtener_fecha_hoy();
            break; // Fecha válida por defecto, salimos del bucle
        } else {
            // Si escribió algo, lo mandamos a verificar a nuestra función de validación
            match utils::validar_formato_fecha(&entrada_fecha) {
                Ok(_) => {
                    fecha_corte_historico = entrada_fecha;
                    break; // Fecha válida e introduce un día real, salimos del bucle
                }
                Err(mensaje_error) => {
                    println!("Error: {}\nInténtalo de nuevo.", mensaje_error);
                }
            }
        }
    }

    // Aquí 'fecha_corte_historico' ya contiene un String garantizado de ser "AAAA-MM-DD" real.
    // El struct Config guarda la fecha como NaiveDate (no como String), así que la convertimos.
    // Usamos .expect() y no un manejo de error porque el String ya fue validado arriba,
    // así que esta conversión no debería fallar nunca.
    utils::validar_formato_fecha(&fecha_corte_historico)
        .expect("La fecha ya había sido validada antes, esto no debería fallar")
}

/// Orquesta todas las preguntas de la configuración inicial, en orden,
/// y arma el struct Config final con todas las respuestas.
pub fn _preguntas_config() -> Config {
    let boveda_path = pregunta_boveda_path();
    let (final_absolute_path, final_relative_path) = pregunta_imagenes_path(&boveda_path);
    let cover_size = pregunta_cover_size();
    let (sistema_calificacion, usar_emojis) = pregunta_sistema_calificacion();
    let preferencia_nombre = pregunta_preferencia_nombre();
    let fecha_corte_historico = pregunta_fecha_corte();
    let config_path = obtener_ruta_carpeta_base();

    // Con todas las respuestas ya recolectadas, armamos el struct Config y lo devolvemos
    // para que quien llamó a esta función (main.rs) pueda pasárselo a rellenar_config().
    Config {
        boveda_path,
        final_absolute_path,
        final_relative_path,
        config_path,
        cover_size,
        sistema_calificacion,
        usar_emojis,
        preferencia_nombre,
        fecha_corte_historico,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub boveda_path: PathBuf,
    pub final_absolute_path: PathBuf,
    pub final_relative_path: PathBuf,
    pub config_path: Option<PathBuf>,
    pub cover_size: u32,
    pub sistema_calificacion: u8,
    pub usar_emojis: bool,
    pub preferencia_nombre: String,
    pub fecha_corte_historico: NaiveDate, // O String si prefieres no validar la fecha formalmente
}

impl Config {
    pub fn validar(&self) -> Result<(), String> {
        // 1. Validar boveda_path: que exista y contenga .obsidian
        if !self.boveda_path.exists() {
            return Err(format!(
                "boveda_path no existe en el disco: {:?}", 
                self.boveda_path
            ));
        }
        let obsidian_dir = self.boveda_path.join(".obsidian");
        if !obsidian_dir.exists() || !obsidian_dir.is_dir() {
            return Err(format!(
                "boveda_path ({:?}) no es una bóveda de Obsidian válida (falta carpeta .obsidian).", 
                self.boveda_path
            ));
        }

        // 2. Validar final_absolute_path y final_relative_path
        if self.final_absolute_path == self.boveda_path {
            return Err("final_absolute_path no puede ser idéntica a boveda_path.".to_string());
        }

        // Verificar que final_absolute_path esté dentro de boveda_path
        let rel_strip = self.final_absolute_path.strip_prefix(&self.boveda_path)
            .map_err(|_| format!(
                "final_absolute_path ({:?}) debe estar contenida dentro de boveda_path ({:?}).",
                self.final_absolute_path, self.boveda_path
            ))?;

        // Verificar que esté por lo menos una carpeta por debajo (no sea la raíz directamente)
        if rel_strip.as_os_str().is_empty() {
            return Err("final_absolute_path debe estar por lo menos una carpeta por debajo de boveda_path.".to_string());
        }

        // Validar final_relative_path: Debe construirse removiendo boveda_path de final_absolute_path
        let expected_relative = Path::new("/").join(rel_strip);
        if self.final_relative_path != expected_relative {
            return Err(format!(
                "final_relative_path ({:?}) no coincide con la versión relativa calculada ({:?}).",
                self.final_relative_path, expected_relative
            ));
        }

        // 3. Validar config_path: debe apuntar a la ruta exacta del programa en el HOME del usuario
        if let Some(ref ruta_pactada) = self.config_path {
            if let Some(ruta_base_esperada) = obtener_ruta_carpeta_base() {
                if ruta_pactada != &ruta_base_esperada {
                    return Err(format!(
                        "config_path incorrecto. Se esperaba: {:?}", 
                        ruta_base_esperada
                    ));
                }
            }
        } else {
            return Err("El campo config_path no puede ser nulo.".to_string());
        }

        // 4. Validar cover_size: debe ser un número entre 50 y 200
        if self.cover_size < 50 || self.cover_size > 200 {
            return Err(format!(
                "cover_size ({}) está fuera de rango. Debe ser un valor entre 50 y 200.", 
                self.cover_size
            ));
        }

        // 5. Validar sistema_calificacion
        if self.sistema_calificacion < 1 || self.sistema_calificacion > 4 {
            return Err(format!(
                "sistema_calificacion ({}) no es válido. Opciones permitidas: 1, 2, 3 o 4.", 
                self.sistema_calificacion
            ));
        }

        // 6. Validar usar_emojis respecto a sistema_calificacion
        if self.usar_emojis && self.sistema_calificacion != 1 {
            return Err(format!(
                "usar_emojis es 'true', pero esto sólo se permite cuando sistema_calificacion es 1 (actualmente es {}).",
                self.sistema_calificacion
            ));
        }

        // 7. Validar preferencia_nombre
        let opciones_validas = ["Propio", "Romaji Japones", "Ingles", "Nativo Japones"];
        if !opciones_validas.contains(&self.preferencia_nombre.as_str()) {
            return Err(format!(
                "preferencia_nombre '{}' no es válida. Valores permitidos: {:?}",
                self.preferencia_nombre, opciones_validas
            ));
        }

        // 8. Validar fecha_corte_historico
        // Ya que el struct deserializa este campo como NaiveDate utilizando serde de chrono,
        // la validez del formato AAAA-MM-DD y la existencia real del día en el calendario
        // quedan garantizadas automáticamente en el proceso de deserialización del JSON. 
        // Si tuviera un formato inválido, el parseo de serde_json fallaría antes de esta validación.

        Ok(())
    }
}

pub fn obtener_ruta_carpeta_base() -> Option<PathBuf> {
    if let Some(mut path) = home::home_dir() {
        path.push(".config");
        path.push("Anime-Obsidean"); // Asegúrate de unificar el guion (-) o bajo (_) según prefieras
        Some(path)
    } else {
        println!("No se pudo determinar el directorio home del usuario.");
        None
    }
}

/// Crea o sobrescribe el archivo config.json con los datos recolectados.
pub fn rellenar_config(config_data: &Config, ruta_guardado: &PathBuf) -> io::Result<()> {
    // Se intenta crear el archivo en la ruta especificada (ej. "config.json")
    // Si el archivo ya existe, lo trunca (borra el contenido previo).
    let mut archivo = File::create(ruta_guardado)?;

    // Convierte la estructura a un String JSON con formato legible (pretty print)
    // El operador ? propagará el error de serialización si algo falla (ej. datos inválidos)
    let json_estructurado = serde_json::to_string_pretty(config_data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Escribe los bytes del string JSON en el archivo
    archivo.write_all(json_estructurado.as_bytes())?;

    // Asegura que todo se haya volcado al disco antes de terminar
    archivo.flush()?;

    Ok(())
}

pub fn verificar_config() -> bool {
    // Verifica si el archivo de configuración existe
    let ruta_config = obtener_ruta_config().unwrap_or_else(|| PathBuf::from("config.json"));
    ruta_config.exists()
}

pub fn cargar_config() -> Result<Config, Box<dyn std::error::Error>> {
    let ruta = obtener_ruta_config()
        .ok_or("No se pudo determinar el directorio HOME.")?;
    
    let archivo = File::open(ruta)?;
    let lector = BufReader::new(archivo);
    let config: Config = serde_json::from_reader(lector)?;
    
    // Ejecutamos la validación lógica y semántica de las reglas de negocio
    config.validar()?;
    
    Ok(config)
}

pub fn obtener_ruta_config() -> Option<PathBuf> {
    // home_dir() devuelve un Option<PathBuf> porque podría fallar si el entorno está roto
    if let Some(mut path) = home::home_dir() {
        // .push() va construyendo la ruta de forma segura añadiendo los separadores / automáticamente
        path.push(".config");
        path.push("Anime-Obsidean");
        path.push("config.json");
        Some(path)
    } else {
        println!("No se pudo determinar el directorio home del usuario.");
        None
    }
}

/// Menú interactivo para editar la configuración existente.
pub fn editar_config() {
    // 1. Cargar la configuración actual
    let mut config = match cargar_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            println!("\nError al cargar la configuración activa: {}", e);
            println!("Presiona Enter para volver al menú principal...");
            let _ = utils::solicitar_entrada("");
            return;
        }
    };

    let mut realizada_modificacion = false;
    let mut necesita_regenerar = false;

    loop {
        limpiar_consola("\n── Panel de Edición de Configuración ──");
        
        let menu = format!(
            "\nSelecciona el parámetro que deseas modificar:\n\n\
            1. Ruta de la bóveda de Obsidian (Actual: {:?})\n\
            2. Ruta de almacenamiento de imágenes (Actual: {:?})\n\
            3. Tamaño de las portadas (Actual: {}px)\n\
            4. Sistema de calificación (Actual: {})\n\
            5. Usar emojis en nombres (Actual: {})\n\
            6. Preferencia de nombre de archivos (Actual: '{}')\n\
            7. Fecha de corte histórico (Actual: {})\n\
            8. Guardar cambios y salir\n\
            9. Cancelar y salir sin guardar\n",
            config.boveda_path,
            config.final_relative_path,
            config.cover_size,
            config.sistema_calificacion,
            config.usar_emojis,
            config.preferencia_nombre,
            config.fecha_corte_historico
        );

        let opcion = utils::solicitar_numero(&menu, 1, 9);

        match opcion {
            1 => {
                // Cambiar ruta de la bóveda
                println!("\nADVERTENCIA:");
                println!("Cambiar la ruta de la bóveda requerirá mover todas las imágenes de portada");
                println!("y volver a generar todos los archivos Markdown (.md) en el nuevo destino.");
                println!("\n¿Estás seguro de que deseas continuar?\n1. Sí\n2. No");
                
                if utils::solicitar_numero("", 1, 2) == 1 {
                    let antigua_boveda = config.boveda_path.clone();
                    let nueva_boveda = pregunta_boveda_path();
                    
                    if nueva_boveda != antigua_boveda {
                        // Calcular nuevas rutas de imágenes basadas en la nueva bóveda
                        let antigua_img_abs = config.final_absolute_path.clone();
                        let nueva_img_abs = nueva_boveda.join(config.final_relative_path.strip_prefix("/").unwrap_or(&config.final_relative_path));

                        // Intentar mover las imágenes físicas
                        if antigua_img_abs.exists() {
                            println!("\nMoviendo imágenes a la nueva ubicación de la bóveda...");
                            if let Err(e) = utils::mover_imagenes(&antigua_img_abs, &nueva_img_abs) {
                            eprintln!("Error al mover las imágenes: {}", e);
                            }
                        }
                        config.boveda_path = nueva_boveda;
                        config.final_absolute_path = nueva_img_abs;
                        
                        realizada_modificacion = true;
                        necesita_regenerar = true;
                        println!("\nRuta de la bóveda actualizada.");
                    }
                }
            }
            2 => {
                // Cambiar almacenamiento de imágenes
                let antigua_img_abs = config.final_absolute_path.clone();
                let (nueva_img_abs, nueva_img_rel) = pregunta_imagenes_path(&config.boveda_path);

                if nueva_img_abs != antigua_img_abs {
                    println!("\nMoviendo imágenes de portada de la antigua ruta a la nueva...");
                    if let Err(e) = utils::mover_imagenes(&antigua_img_abs, &nueva_img_abs) {
                    eprintln!("Error al mover las imágenes: {}", e);
                }

                    // Borrar la carpeta vieja si quedó vacía o si se requiere eliminar por flujo
                    if antigua_img_abs.exists() {
                        let _ = remove_dir_all(&antigua_img_abs);
                        println!("Carpeta de imágenes antigua eliminada.");
                    }

                    config.final_absolute_path = nueva_img_abs;
                    config.final_relative_path = nueva_img_rel;

                    realizada_modificacion = true;
                    necesita_regenerar = true;
                    println!("\nUbicación de almacenamiento de imágenes actualizada con éxito.");
                }
            }
            3 => {
                let nuevo_tamano = pregunta_cover_size();
                if nuevo_tamano != config.cover_size {
                    config.cover_size = nuevo_tamano;
                    realizada_modificacion = true;
                    necesita_regenerar = true;
                }
            }
            4 => {
                let (nuevo_sistema, nuevo_emoji) = pregunta_sistema_calificacion();
                if nuevo_sistema != config.sistema_calificacion || nuevo_emoji != config.usar_emojis {
                    config.sistema_calificacion = nuevo_sistema;
                    config.usar_emojis = nuevo_emoji;
                    realizada_modificacion = true;
                    necesita_regenerar = true;
                }
            }
            5 => {
                if config.sistema_calificacion != 1 {
                    println!("\nError: Solo puedes usar emojis si el sistema de calificación es 'Por gustos' (Opción 1).");
                    println!("Modifica primero el sistema de calificación.");
                    let _ = utils::solicitar_entrada("\nPresiona Enter para continuar...");
                } else {
                    let menu_emoji = String::from("\n¿Quieres usar emojis en los nombres?\n1. Sí\n2. No\n");
                    let seleccion = utils::solicitar_numero(&menu_emoji, 1, 2);
                    let usar_emoji_bool = seleccion == 1;

                    if usar_emoji_bool != config.usar_emojis {
                        config.usar_emojis = usar_emoji_bool;
                        realizada_modificacion = true;
                        necesita_regenerar = true;
                    }
                }
            }
            6 => {
                let nuevo_nombre = pregunta_preferencia_nombre();
                if nuevo_nombre != config.preferencia_nombre {
                    config.preferencia_nombre = nuevo_nombre;
                    realizada_modificacion = true;
                    necesita_regenerar = true;
                }
            }
            7 => {
                let nueva_fecha = pregunta_fecha_corte();
                if nueva_fecha != config.fecha_corte_historico {
                    config.fecha_corte_historico = nueva_fecha;
                    realizada_modificacion = true;
                    necesita_regenerar = true;
                }
            }
            8 => {
                // Guardar cambios y aplicar regeneraciones
                if realizada_modificacion {
                    if let Some(ruta_guardado) = obtener_ruta_config() {
                        // Validar estructuralmente antes de escribir en disco
                        if let Err(err_val) = config.validar() {
                            println!("\nLa nueva configuración no es válida de acuerdo a las reglas: {}", err_val);
                            println!("Corrige los campos antes de guardar.");
                            let _ = utils::solicitar_entrada("\nPresiona Enter para continuar...");
                            continue;
                        }

                        match rellenar_config(&config, &ruta_guardado) {
                            Ok(()) => {
                                println!("\nConfiguración guardada exitosamente en el archivo.");
                                if necesita_regenerar {
                                    println!("Aplicando cambios estructurales: Regenerando archivos Markdown (.md)...");
                                    // Invocar aquí la función del sistema para reconstruir la bóveda
                                    utils::regenerar_md();
                                }
                            }
                            Err(e) => eprintln!("Error al escribir el archivo de configuración: {}", e),
                        }
                    }
                } else {
                    println!("\nNo se detectaron cambios que guardar.");
                }
                break;
            }
            9 => {
                println!("\nCambios descartados. Volviendo al menú principal...");
                break;
            }
            _ => unreachable!(),
        }
    }
}