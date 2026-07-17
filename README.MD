# Estructura de proyecto

[ Interfaz CLI ] 
       │
       ▼
┌──────────────────────────┐
│  Se carga (config.json)  │ ──> Si no existe se hacen las preguntas de configuración
└──────────────────────────┘
       | 
       | 
       ▼ (Se muestra la interfaz para el tipo de entrada)
[ En caso de selecionar anime ] ───> Con el nombre se obtiene la ID de anilist
       |                                       |
       |                                 ¿El anime ya esta registrado?
       |                                       |
       │                                       ├─ SÍ: Editar campos.
       │                 ┌─────────────────────└─ NO: Crear nueva entrada.
       ▼                 ▼
┌──────────────────────────────────────┐
│  BASE DE DATOS CENTRAL (data.json)   │
└──────────────────────────────────────┘
       │
       └─► [ Módulo Generador MD ] ──► [ Carpeta Obsidian (.md) ]

# Flujo Pincipal
1- Se inicia la app CLI y se revisa las configuraciones.
2- Selector de opciones
3- Se muestran las preguntas principales del script
4- Se solicita a la API toda la info y se descarga la imagen.
5- Se crea la nueva entrada en el json en base a la plantilla
6- Se crea tambien el nuevo .md en base a la nueva entrada del data.json y a la plantilla

## Opciones
1- Anime
       1.1- Crear nueva entrada
       1.2- Añadir nuevo contenido visto a entrada existente
       1.3- Elimina todas las entradas de anime y las vuelve a crear en base a data.json
       1.4- Crea una version comprimida del toda la base de datos, para pasarle a una IA o simplemente leerlo
       1.5- Se añade una simple recomendación o anime que se quiere ver
2- Manga
       2.1- No pienso crear este apartado todavia.
3- Libros
       3.1- No pienso crear este apartado todavia.

## Configuraciones
En la configuracion de la app se guardara en ~/.config/Anime-Obsidean/config.json

Dentro de config.json se guardara lo siguiente:
 
1- Ruta de la boveda de Obsidean.
2- Ruta interna de obsidean donde se guardaran las imagenes.
3- Tamaño de las imagenes del dataviewjs. (Por defecto 90)
4- Tipos de clasificación, del 1 al 10, del 1 al 5, tipos, sin clasificacion.
5- Si queres o no usar emojis en los nombres (Solo compatible con tipo de clasificacion Tipos).
7- Preferis nombres en Romanji en ingles o los nombres propios.
8- Idioma por defecto selecionado para las preguntas.