use std::process::id;

use crate::config;
use crate::utils;
use serde::{Deserialize};
use serde_json::json;
use strsim::levenshtein;

/// usar config_data en modo box carga la estructura general del config.json ademas de que llama a todas las preguntas de los animes
pub async fn preguntas_anime(config_data: config::Config) {
    println!("{} se guardo todo correctamente", config_data.preferencia_nombre);

    nombre_anime().await;
}

/// Apartir de aqui comienza la funcion para hacer la solicitud a la API de anilist y buscar animes.
#[derive(Deserialize, Debug)]
struct AnilistTitle {
    #[serde(rename = "userPreferred")]
    user_preferred: Option<String>,
    english: Option<String>,
    romaji: Option<String>,
}

#[derive(Deserialize, Debug)]
struct AnimeMedia {
    id: u32,
    title: AnilistTitle,
}

#[derive(Deserialize, Debug)]
struct Page {
    media: Vec<AnimeMedia>,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    data: Option<PageData>,
}

#[derive(Deserialize, Debug)]
struct PageData {
    #[serde(rename = "Page")]
    page: Page,
}

/// Busca un anime en AniList y devuelve la ID de la mejor coincidencia.
async fn get_best_anime_match(search_term: &str) -> Result<Option<u32>, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = "https://graphql.anilist.co";

    let query = r#"
        query ($search: String) {
          Page(page: 1, perPage: 6) {
            media(search: $search, type: ANIME) {
              id
              title {
                userPreferred
                english
                romaji
              }
            }
          }
        }
    "#;

    // Construcción del cuerpo de la solicitud GraphQL
    let body = json!({
        "query": query,
        "variables": {
            "search": search_term
        }
    });

    let response: ApiResponse = client
        .post(url)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    let media_list = match response.data {
        Some(d) => d.page.media,
        None => return Ok(None),
    };

    if media_list.is_empty() {
        return Ok(None);
    }

    let search_lower = search_term.to_lowercase();
    let mut best_id = None;
    let mut min_distance = usize::MAX;

    // Evaluación de la mejor coincidencia basada en la distancia de Levenshtein
    for anime in media_list {
        let titles = vec![
            anime.title.user_preferred.as_ref(),
            anime.title.english.as_ref(),
            anime.title.romaji.as_ref(),
        ];

        for title in titles.into_iter().flatten() {
            let title_lower = title.to_lowercase();
            let distance = levenshtein(&search_lower, &title_lower);

            // Si encontramos una coincidencia exacta, terminamos temprano
            if distance == 0 {
                return Ok(Some(anime.id));
            }

            if distance < min_distance {
                min_distance = distance;
                best_id = Some(anime.id);
            }
        }
    }

    Ok(best_id)
}

async fn nombre_anime() {

    loop {
        utils::limpiar_consola("— Añadir ANIME —");

        let search = &utils::solicitar_entrada("— Ingresa el nombre del ANIME —");
        match get_best_anime_match(search).await {
        Ok(Some(id)) => {solicitar_info(id).await;
        break
        },
        Ok(None) => {println!("No se encontraron resultados para '{}'.", search);
        continue
        },
        Err(e) => {eprintln!("Error al realizar la solicitud: {}", e);
        continue;
        },
      }
    }
}

async fn solicitar_info(id: u32) {

} 