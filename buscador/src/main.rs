//! Buscador de archivos por nombre dentro de la carpeta del usuario.
//!
//! Flujo: pregunta -> búsqueda -> resultados -> salir o continuar.
//! Sólo dependencias de la biblioteca estándar.

use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::Instant;

/// Hilos que recorren el disco en paralelo (la búsqueda es E/S, no CPU).
const MAX_HILOS: usize = 16;

#[derive(Clone, Copy, PartialEq)]
enum Modo {
    Exacto,
    Contiene,
}

impl Modo {
    /// Devuelve el nombre del modo para mostrarlo en pantalla.
    fn etiqueta(self) -> &'static str {
        match self {
            Modo::Exacto => "coincidencia exacta",
            Modo::Contiene => "contiene",
        }
    }

    /// Indica si el nombre de un archivo casa con el patrón según el modo:
    /// igualdad completa en `Exacto`, subcadena en `Contiene`. Ambos textos
    /// deben llegar ya en minúsculas.
    fn coincide(self, nombre_min: &str, patron_min: &str) -> bool {
        match self {
            Modo::Exacto => nombre_min == patron_min,
            Modo::Contiene => nombre_min.contains(patron_min),
        }
    }
}

struct Resumen {
    directorios: usize,
    archivos: usize,
    inaccesibles: usize,
}

/// Punto de entrada: localiza la carpeta del usuario, muestra la cabecera y
/// ejecuta el bucle principal (pedir patrón -> pedir modo -> buscar ->
/// imprimir resultados -> preguntar si se continúa) hasta que se pide salir.
fn main() {
    let raiz = match carpeta_usuario() {
        Some(p) => p,
        None => {
            eprintln!("No se pudo determinar la carpeta del usuario (USERPROFILE/HOME).");
            std::process::exit(1);
        }
    };

    println!();
    println!("=============================================");
    println!("  Buscador de archivos");
    println!("=============================================");
    println!("  Carpeta base : {}", raiz.display());
    println!("  Alcance      : todos los subdirectorios");
    println!("  Busca sólo archivos (las carpetas se ignoran)");
    println!("  Sin distinción entre mayúsculas y minúsculas");
    println!("  Escribe :q en cualquier momento para salir");
    println!();

    loop {
        let patron = match pedir_patron() {
            Some(p) => p,
            None => break,
        };

        let modo = match pedir_modo() {
            Some(m) => m,
            None => break,
        };

        println!();
        println!(
            "Buscando \"{}\" ({}) en {} ...",
            patron,
            modo.etiqueta(),
            raiz.display()
        );

        let inicio = Instant::now();
        let (resultados, resumen) = buscar(&raiz, &patron.to_lowercase(), modo);
        let duracion = inicio.elapsed();

        println!();
        if resultados.is_empty() {
            println!("Sin resultados: ningún archivo coincide con \"{}\".", patron);
        } else {
            println!("--- Resultados ({}) ---", resultados.len());
            let ancho = resultados.len().to_string().len();
            for (i, ruta) in resultados.iter().enumerate() {
                println!("{:>ancho$}. {}", i + 1, ruta.display(), ancho = ancho);
            }
        }

        println!();
        println!(
            "{} archivo(s) encontrado(s) | {} archivos revisados en {} carpetas | {:.2} s",
            resultados.len(),
            resumen.archivos,
            resumen.directorios,
            duracion.as_secs_f64()
        );
        if resumen.inaccesibles > 0 {
            println!(
                "({} carpeta(s) omitida(s) por falta de permisos)",
                resumen.inaccesibles
            );
        }
        println!();

        if !menu_final(&resultados) {
            break;
        }
    }

    println!("Hasta luego.");
}

/// Carpetas pendientes de recorrer, compartidas por los hilos.
struct Cola {
    estado: Mutex<Estado>,
    aviso: Condvar,
}

struct Estado {
    pendientes: Vec<PathBuf>,
    /// Hilos que están recorriendo una carpeta ahora mismo.
    ocupados: usize,
    fin: bool,
}

/// Recorre la carpeta base y toda su descendencia con varios hilos y devuelve
/// las rutas de los archivos que coinciden (ordenadas alfabéticamente) junto
/// con el resumen del recorrido.
fn buscar(raiz: &Path, patron_min: &str, modo: Modo) -> (Vec<PathBuf>, Resumen) {
    let cola = Cola {
        estado: Mutex::new(Estado {
            pendientes: vec![raiz.to_path_buf()],
            ocupados: 0,
            fin: false,
        }),
        aviso: Condvar::new(),
    };
    let directorios = AtomicUsize::new(0);
    let archivos = AtomicUsize::new(0);
    let inaccesibles = AtomicUsize::new(0);
    let hallados: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

    let hilos = std::thread::available_parallelism()
        .map(|n| n.get() * 2)
        .unwrap_or(4)
        .clamp(2, MAX_HILOS);

    std::thread::scope(|ambito| {
        for _ in 0..hilos {
            ambito.spawn(|| {
                let mut locales = Vec::new();
                while let Some(dir) = siguiente(&cola) {
                    recorrer(
                        &dir,
                        patron_min,
                        modo,
                        &cola,
                        &directorios,
                        &archivos,
                        &inaccesibles,
                        &mut locales,
                    );
                }
                if !locales.is_empty() {
                    hallados.lock().unwrap().append(&mut locales);
                }
            });
        }
    });

    limpiar_linea();
    let mut resultados = hallados.into_inner().unwrap();
    resultados.sort();
    let resumen = Resumen {
        directorios: directorios.load(Ordering::Relaxed),
        archivos: archivos.load(Ordering::Relaxed),
        inaccesibles: inaccesibles.load(Ordering::Relaxed),
    };
    (resultados, resumen)
}

/// Entrega a un hilo la siguiente carpeta pendiente; si no hay ninguna
/// disponible espera a que otro hilo aporte más y devuelve `None` sólo cuando
/// ya no queda trabajo en ningún hilo (fin del recorrido).
fn siguiente(cola: &Cola) -> Option<PathBuf> {
    let mut estado = cola.estado.lock().unwrap();
    loop {
        if let Some(dir) = estado.pendientes.pop() {
            estado.ocupados += 1;
            return Some(dir);
        }
        if estado.fin || estado.ocupados == 0 {
            estado.fin = true;
            cola.aviso.notify_all();
            return None;
        }
        estado = cola.aviso.wait(estado).unwrap();
    }
}

/// Examina una sola carpeta: guarda en `locales` los archivos que coinciden,
/// encola sus subcarpetas para el resto de hilos y actualiza los contadores.
#[allow(clippy::too_many_arguments)]
fn recorrer(
    dir: &Path,
    patron_min: &str,
    modo: Modo,
    cola: &Cola,
    directorios: &AtomicUsize,
    archivos: &AtomicUsize,
    inaccesibles: &AtomicUsize,
    locales: &mut Vec<PathBuf>,
) {
    let mut subcarpetas = Vec::new();
    let mut vistos = 0usize;

    match fs::read_dir(dir) {
        Ok(entradas) => {
            let previos = directorios.fetch_add(1, Ordering::Relaxed);
            if previos % 500 == 0 {
                progreso(previos + 1, archivos.load(Ordering::Relaxed));
            }

            for entrada in entradas.flatten() {
                // file_type() no sigue enlaces: evita ciclos por symlinks y junctions.
                let tipo = match entrada.file_type() {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                if tipo.is_symlink() {
                    continue;
                }

                if tipo.is_dir() {
                    subcarpetas.push(entrada.path());
                    continue;
                }

                vistos += 1;
                let nombre = entrada.file_name();
                let nombre_min = nombre.to_string_lossy().to_lowercase();
                if modo.coincide(&nombre_min, patron_min) {
                    locales.push(entrada.path());
                }
            }
            archivos.fetch_add(vistos, Ordering::Relaxed);
        }
        Err(_) => {
            inaccesibles.fetch_add(1, Ordering::Relaxed);
        }
    }

    let mut estado = cola.estado.lock().unwrap();
    let hay_nuevas = !subcarpetas.is_empty();
    estado.pendientes.append(&mut subcarpetas);
    estado.ocupados -= 1;
    if hay_nuevas || estado.ocupados == 0 {
        cola.aviso.notify_all();
    }
}

/// Reescribe en la misma línea el avance del recorrido mientras se busca.
fn progreso(directorios: usize, archivos: usize) {
    print!(
        "\r  ... {} carpetas / {} archivos revisados",
        directorios, archivos
    );
    let _ = io::stdout().flush();
}

/// Borra la línea de progreso para que no ensucie los resultados.
fn limpiar_linea() {
    print!("\r{}\r", " ".repeat(60));
    let _ = io::stdout().flush();
}

/// Pide el nombre a buscar, insistiendo mientras la respuesta esté vacía.
/// Devuelve `None` si el usuario decide salir.
fn pedir_patron() -> Option<String> {
    loop {
        let entrada = leer("Nombre del archivo a buscar: ")?;
        let entrada = entrada.trim();
        if es_salida(entrada) {
            return None;
        }
        if entrada.is_empty() {
            println!("  Escribe algo (o :q para salir).");
            continue;
        }
        return Some(entrada.to_string());
    }
}

/// Pide el tipo de coincidencia (exacta o contiene), repitiendo el menú ante
/// una opción no válida. Devuelve `None` si el usuario decide salir.
fn pedir_modo() -> Option<Modo> {
    loop {
        println!("  [1] Coincidencia exacta   [2] Que contenga el texto");
        let entrada = leer("Modo de búsqueda [1/2] (Enter = 2): ")?;
        let entrada = entrada.trim();
        if es_salida(entrada) {
            return None;
        }
        match entrada {
            "1" | "e" | "E" => return Some(Modo::Exacto),
            "" | "2" | "c" | "C" => return Some(Modo::Contiene),
            _ => println!("  Opción no válida."),
        }
    }
}

/// Muestra el menú posterior a la búsqueda (repetir, guardar los resultados en
/// un archivo o terminar). Devuelve `true` si se debe hacer otra búsqueda.
fn menu_final(resultados: &[PathBuf]) -> bool {
    loop {
        let opcion = match leer("¿Otra búsqueda? [s]í / [g]uardar resultados / [n]o: ") {
            Some(o) => o,
            None => return false,
        };
        match opcion.trim() {
            "" | "s" | "S" | "si" | "sí" | "Si" | "SI" => return true,
            "n" | "N" | "no" | "No" | "NO" | ":q" | "q" | "salir" => return false,
            "g" | "G" => {
                guardar(resultados);
                println!();
            }
            _ => println!("  Opción no válida."),
        }
    }
}

/// Pide una ruta de destino y escribe en ella las rutas encontradas, una por
/// línea. Informa por pantalla del resultado en lugar de abortar si falla.
fn guardar(resultados: &[PathBuf]) {
    if resultados.is_empty() {
        println!("  No hay resultados que guardar.");
        return;
    }
    let destino = match leer("  Ruta del archivo de salida (Enter = resultados.txt): ") {
        Some(d) => {
            let d = d.trim().to_string();
            if d.is_empty() {
                "resultados.txt".to_string()
            } else {
                d
            }
        }
        None => return,
    };

    match File::create(&destino) {
        Ok(archivo) => {
            let mut escritor = BufWriter::new(archivo);
            let mut error = None;
            for ruta in resultados {
                if let Err(e) = writeln!(escritor, "{}", ruta.display()) {
                    error = Some(e);
                    break;
                }
            }
            match error.or_else(|| escritor.flush().err()) {
                Some(e) => println!("  No se pudo escribir: {}", e),
                None => println!("  Guardado en {} ({} rutas).", destino, resultados.len()),
            }
        }
        Err(e) => println!("  No se pudo crear \"{}\": {}", destino, e),
    }
}

/// Indica si lo escrito es uno de los comandos para salir del programa.
fn es_salida(entrada: &str) -> bool {
    matches!(entrada, ":q" | ":Q" | ":salir" | ":exit")
}

/// Muestra un prompt y lee una línea del teclado. Devuelve `None` si se cierra
/// la entrada (Ctrl+Z / Ctrl+D).
fn leer(prompt: &str) -> Option<String> {
    print!("{}", prompt);
    let _ = io::stdout().flush();
    let mut buffer = String::new();
    match io::stdin().read_line(&mut buffer) {
        Ok(0) => {
            println!();
            None
        }
        // Se descarta el BOM que algunas consolas anteponen a la primera línea.
        Ok(_) => Some(buffer.trim_start_matches('\u{feff}').to_string()),
        Err(_) => None,
    }
}

/// Localiza la carpeta del usuario a partir de USERPROFILE o HOME, con
/// `home_dir()` como último recurso. Devuelve `None` si no se puede determinar.
fn carpeta_usuario() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .or_else(|| {
            #[allow(deprecated)]
            std::env::home_dir()
        })
}
