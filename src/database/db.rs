use rusqlite::Connection;
use crate::config::obtener_ruta_db;

pub fn open_connection() -> Result<Connection, Box<dyn std::error::Error>> {
    let ruta = obtener_ruta_db().ok_or("No se pudo determinar la ruta de la base de datos")?;

    // Crea la carpeta si no existe (si ya existe, no hace nada)
    if let Some(carpeta) = ruta.parent() {
        std::fs::create_dir_all(carpeta)?;
    }

    let conn = Connection::open(ruta)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(conn)
}

pub fn create_database() -> Result<(), Box<dyn std::error::Error>> {
    let conn = open_connection()?;
 
    // 1. user_library
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_library (
            id_local INTEGER PRIMARY KEY AUTOINCREMENT,
            media_type TEXT NOT NULL,
            like_user INTEGER CHECK(like_user IN (-1, 0, 1)),
            score INTEGER CHECK(score BETWEEN 10 AND 50),
            favorite BOOLEAN NOT NULL DEFAULT 0 CHECK(favorite IN (0, 1)),
            add_db_date TEXT NOT NULL,
            state TEXT NOT NULL DEFAULT 'viendo' CHECK(state IN ('viendo', 'finalizado', 'abandonado', 'pausado', 'por_ver', 'al_dia')),
            rewatch_count INTEGER NOT NULL DEFAULT 0 CHECK(rewatch_count >= 0),
            opinion TEXT,
            volume INTEGER CHECK(volume >= 0),
            chapter INTEGER CHECK(chapter >= 0),
            is_archived BOOLEAN NOT NULL DEFAULT 0 CHECK(is_archived IN (0, 1)),
            title_user TEXT NOT NULL
        )",
        [],
    )?;
 
    // 2. media_info
    // Cambios: title_synonyms debe ser JSON válido (o NULL) y
    // UNIQUE(id_API, API_name) evita duplicar la misma obra de la misma API.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS media_info (
            id_media INTEGER PRIMARY KEY AUTOINCREMENT,
            id_local INTEGER NOT NULL REFERENCES user_library(id_local) ON DELETE CASCADE,
            id_API INTEGER NOT NULL,
            API_name TEXT NOT NULL,
            title_english TEXT,
            title_spanish TEXT,
            title_romaji TEXT,
            title_native TEXT,
            user_preferred TEXT,
            title_synonyms TEXT CHECK(title_synonyms IS NULL OR json_valid(title_synonyms)),
            description TEXT,
            cover_site_url TEXT,
            cover_path TEXT,
            cover_color TEXT,
            banner_site_url TEXT,
            banner_path TEXT,
            format TEXT NOT NULL,
            status TEXT NOT NULL,
            chapters INTEGER CHECK(chapters >= 0),
            volumes INTEGER CHECK(volumes >= 0),
            season TEXT,
            season_year INTEGER CHECK(season_year >= 0),
            source TEXT,
            is_adult BOOLEAN NOT NULL CHECK(is_adult IN (0, 1)),
            country_of_origin TEXT,
            site_url TEXT,
            start_date TEXT,
            end_date TEXT,
            genres TEXT,
            cache_last_updated TEXT,
            cache_synced BOOLEAN NOT NULL DEFAULT 0 CHECK(cache_synced IN (0, 1)),
            translate_synopsis TEXT,
            is_root BOOLEAN NOT NULL DEFAULT 0 CHECK(is_root IN (0, 1)),
            UNIQUE(id_API, API_name)
        )",
        [],
    )?;
 
    // 3. tags
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id_tag INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            category TEXT,
            rank INTEGER CHECK(rank >= 0)
        )",
        [],
    )?;
 
    // 4. media_tags
    conn.execute(
        "CREATE TABLE IF NOT EXISTS media_tags (
            id_local INTEGER NOT NULL REFERENCES user_library(id_local) ON DELETE CASCADE,
            id_tag INTEGER NOT NULL REFERENCES tags(id_tag) ON DELETE CASCADE,
            is_general_spoiler BOOLEAN NOT NULL DEFAULT 0 CHECK(is_general_spoiler IN (0, 1)),
            is_media_spoiler BOOLEAN NOT NULL DEFAULT 0 CHECK(is_media_spoiler IN (0, 1)),
            PRIMARY KEY (id_local, id_tag)
        )",
        [],
    )?;
 
    // 5. relation_tree
    conn.execute(
        "CREATE TABLE IF NOT EXISTS relation_tree (
            id_source_media INTEGER NOT NULL REFERENCES media_info(id_media) ON DELETE CASCADE,
            id_target_media INTEGER NOT NULL REFERENCES media_info(id_media) ON DELETE CASCADE,
            relation_type TEXT NOT NULL,
            PRIMARY KEY (id_source_media, id_target_media)
        )",
        [],
    )?;
 
    // 6. watched_content
    conn.execute(
        "CREATE TABLE IF NOT EXISTS watched_content (
            id_local INTEGER NOT NULL REFERENCES user_library(id_local) ON DELETE CASCADE,
            id_media INTEGER NOT NULL REFERENCES media_info(id_media) ON DELETE CASCADE,
            note TEXT,
            start_date TEXT,
            end_date TEXT,
            watched_volumes INTEGER CHECK(watched_volumes >= 0),
            watched_chapters INTEGER CHECK(watched_chapters >= 0),
            PRIMARY KEY (id_local, id_media)
        )",
        [],
    )?;
 
    // 7. Índices (aceleran búsquedas por estas columnas)
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_media_info_local ON media_info(id_local);
         CREATE INDEX IF NOT EXISTS idx_media_tags_tag ON media_tags(id_tag);
         CREATE INDEX IF NOT EXISTS idx_watched_content_media ON watched_content(id_media);
         CREATE INDEX IF NOT EXISTS idx_relation_tree_target ON relation_tree(id_target_media);",
    )?;
 
    // 8. search_index (FTS5)
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
            id_local UNINDEXED,
            id_media UNINDEXED,
            names,
            tokenize = 'unicode61 remove_diacritics 2'
        )",
        [],
    )?;
 
    // 9. Triggers del buscador
 
    // Al insertar una obra: agrega sus nombres al buscador.
    // (Se quitó el JOIN innecesario con user_library.)
    conn.execute_batch(
        "CREATE TRIGGER IF NOT EXISTS trg_search_index_insert
        AFTER INSERT ON media_info
        BEGIN
            INSERT INTO search_index(id_local, id_media, names)
            VALUES (
                NEW.id_local,
                NEW.id_media,
                COALESCE(NEW.title_english, '') || ' ' ||
                COALESCE(NEW.title_spanish, '') || ' ' ||
                COALESCE(NEW.title_romaji, '') || ' ' ||
                COALESCE(NEW.user_preferred, '') || ' ' ||
                COALESCE((
                    SELECT group_concat(value, ' ')
                    FROM json_each(NEW.title_synonyms)
                ), '')
            );
        END;",
    )?;
 
    // NUEVO: al cambiar algún título, actualiza el buscador.
    conn.execute_batch(
        "CREATE TRIGGER IF NOT EXISTS trg_search_index_update
        AFTER UPDATE OF title_english, title_spanish, title_romaji, user_preferred, title_synonyms
        ON media_info
        BEGIN
            UPDATE search_index
            SET names =
                COALESCE(NEW.title_english, '') || ' ' ||
                COALESCE(NEW.title_spanish, '') || ' ' ||
                COALESCE(NEW.title_romaji, '') || ' ' ||
                COALESCE(NEW.user_preferred, '') || ' ' ||
                COALESCE((
                    SELECT group_concat(value, ' ')
                    FROM json_each(NEW.title_synonyms)
                ), '')
            WHERE id_media = NEW.id_media;
        END;",
    )?;
 
    // Al borrar una obra de media_info: la saca del buscador.
    conn.execute_batch(
        "CREATE TRIGGER IF NOT EXISTS trg_search_index_delete_media
        AFTER DELETE ON media_info
        BEGIN
            DELETE FROM search_index WHERE id_media = OLD.id_media;
        END;",
    )?;
 
    // Se mantiene por si algún día borrás de user_library.
    conn.execute_batch(
        "CREATE TRIGGER IF NOT EXISTS trg_search_index_delete
        AFTER DELETE ON user_library
        BEGIN
            DELETE FROM search_index WHERE id_local = OLD.id_local;
        END;",
    )?;
 
    Ok(())
}