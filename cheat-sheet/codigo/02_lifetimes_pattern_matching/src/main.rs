//! Ejemplos ejecutables de lifetimes, tipos algebraicos (structs/enums) y pattern matching.
//! Ver la explicacion conceptual en `cheat-sheet/conceptos/02-lifetimes-pattern-matching.md`.

fn main() {
    ejemplo_lifetime_en_funcion();
    ejemplo_lifetime_en_struct();
    ejemplo_option();
    ejemplo_result();
    ejemplo_match_exhaustivo();
    ejemplo_if_let_while_let();
    ejemplo_match_y_ownership();
}

// ---------- Parte A: lifetimes ----------

/// Dos parametros de referencia: el compilador no puede inferir de cual
/// depende la salida, asi que el lifetime `'a` se anota explicitamente.
/// `'a` es un contrato: "el resultado vive, como minimo, tanto como el
/// mas corto entre x e y" -- no altera cuanto viven x o y.
fn mas_largo<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn ejemplo_lifetime_en_funcion() {
    println!("--- lifetime en funcion ---");
    let s1 = String::from("larga cadena de texto");
    let resultado;
    {
        let s2 = String::from("corta");
        resultado = mas_largo(s1.as_str(), s2.as_str());
        println!("el mas largo es: {resultado}");
        // Si moviéramos el println! fuera de este bloque, el compilador
        // rechazaría el codigo: `resultado` podria depender de `s2`,
        // que ya no viviria fuera de este scope.
    }
}

/// Un struct que guarda una referencia necesita anotar el lifetime:
/// garantiza que ninguna instancia de `Extracto` puede sobrevivir a los
/// datos que referencia.
struct Extracto<'a> {
    parte: &'a str,
}

fn ejemplo_lifetime_en_struct() {
    println!("--- lifetime en struct ---");
    let novela = String::from("Llamame Ismael. Hace algunos anhos...");
    let primera_oracion = novela.split('.').next().unwrap();
    let extracto = Extracto { parte: primera_oracion };
    println!("extracto: '{}'", extracto.parte);
}

// ---------- Parte B: tipos algebraicos y pattern matching ----------

/// Tipo suma: un valor de tipo `Forma` es *exactamente una* de estas
/// variantes, cada una con sus propios datos asociados.
enum Forma {
    Circulo { radio: f64 },
    Rectangulo { ancho: f64, alto: f64 },
    Triangulo { base: f64, altura: f64 },
}

fn area(forma: &Forma) -> f64 {
    match forma {
        Forma::Circulo { radio } => std::f64::consts::PI * radio * radio,
        Forma::Rectangulo { ancho, alto } => ancho * alto,
        Forma::Triangulo { base, altura } => 0.5 * base * altura,
    }
}

fn describe(forma: &Forma) -> String {
    match forma {
        Forma::Circulo { radio } => format!("circulo de radio {radio}"),
        Forma::Rectangulo { ancho, alto } => format!("rectangulo {ancho}x{alto}"),
        Forma::Triangulo { base, altura } => format!("triangulo base {base} altura {altura}"),
    }
}

fn ejemplo_match_exhaustivo() {
    println!("--- match exhaustivo sobre un enum ---");
    let formas = vec![
        Forma::Circulo { radio: 2.0 },
        Forma::Rectangulo { ancho: 3.0, alto: 4.0 },
        Forma::Triangulo { base: 5.0, altura: 6.0 },
    ];
    for forma in &formas {
        println!("{}: area = {:.2}", describe(forma), area(forma));
    }
}

/// `Option<T>` reemplaza a `null`: el compilador obliga a manejar `None`
/// antes de poder usar el valor interior.
fn busca_par(numeros: &[i32]) -> Option<i32> {
    for &n in numeros {
        if n % 2 == 0 {
            return Some(n);
        }
    }
    None
}

fn ejemplo_option() {
    println!("--- Option<T> ---");
    let numeros = [1, 3, 5, 8, 9];
    match busca_par(&numeros) {
        Some(par) => println!("primer par encontrado: {par}"),
        None => println!("no hay pares"),
    }

    let sin_pares = [1, 3, 5];
    let valor = busca_par(&sin_pares).unwrap_or(-1);
    println!("valor con default: {valor}");
    assert_eq!(valor, -1);
}

/// `Result<T, E>` modela errores como valores. La firma de la funcion
/// documenta que la operacion puede fallar.
#[derive(Debug, PartialEq)]
enum ErrorDivision {
    DivisionPorCero,
}

fn divide(a: f64, b: f64) -> Result<f64, ErrorDivision> {
    if b == 0.0 {
        Err(ErrorDivision::DivisionPorCero)
    } else {
        Ok(a / b)
    }
}

fn ejemplo_result() {
    println!("--- Result<T, E> ---");
    match divide(10.0, 2.0) {
        Ok(resultado) => println!("10 / 2 = {resultado}"),
        Err(e) => println!("error: {e:?}"),
    }
    match divide(10.0, 0.0) {
        Ok(resultado) => println!("10 / 0 = {resultado}"),
        Err(e) => println!("error: {e:?}"),
    }
}

/// `if let` / `while let` son azucar sintactico para pattern matching
/// cuando solo importa un patron.
fn ejemplo_if_let_while_let() {
    println!("--- if let / while let ---");
    let config_max: Option<u8> = Some(3);
    if let Some(max) = config_max {
        println!("maximo configurado: {max}");
    }

    let mut pila = vec![1, 2, 3];
    while let Some(top) = pila.pop() {
        println!("desapilado: {top}");
    }
    assert!(pila.is_empty());
}

/// El brazo de un `match` decide, siguiendo las reglas de ownership,
/// si el contenido se presta (`ref`) o se mueve.
fn ejemplo_match_y_ownership() {
    println!("--- match y ownership ---");
    let opcional = Some(String::from("hola"));

    match &opcional {
        Some(s) => println!("prestado via &opcional: {s}"),
        None => {}
    }
    // opcional sigue siendo valido porque hicimos match sobre &opcional
    println!("opcional sigue vivo: {opcional:?}");

    match opcional {
        Some(s) => println!("movido: {s}"),
        None => {}
    }
    // aqui `opcional` ya fue consumido por el match anterior
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mas_largo_devuelve_la_cadena_mas_larga() {
        assert_eq!(mas_largo("abc", "de"), "abc");
        assert_eq!(mas_largo("a", "bcdef"), "bcdef");
    }

    #[test]
    fn area_circulo() {
        let f = Forma::Circulo { radio: 1.0 };
        assert!((area(&f) - std::f64::consts::PI).abs() < 1e-9);
    }

    #[test]
    fn area_rectangulo() {
        let f = Forma::Rectangulo { ancho: 3.0, alto: 4.0 };
        assert_eq!(area(&f), 12.0);
    }

    #[test]
    fn busca_par_encuentra_el_primero() {
        assert_eq!(busca_par(&[1, 3, 4, 5]), Some(4));
    }

    #[test]
    fn busca_par_devuelve_none_si_no_hay() {
        assert_eq!(busca_par(&[1, 3, 5]), None);
    }

    #[test]
    fn divide_normal() {
        assert_eq!(divide(10.0, 2.0), Ok(5.0));
    }

    #[test]
    fn divide_por_cero_es_err() {
        assert_eq!(divide(1.0, 0.0), Err(ErrorDivision::DivisionPorCero));
    }
}
