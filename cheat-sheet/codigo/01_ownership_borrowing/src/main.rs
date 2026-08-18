//! Ejemplos ejecutables de ownership, move, clone, borrowing y slices.
//! Ver la explicacion conceptual en `cheat-sheet/conceptos/01-ownership-borrowing-referencias.md`.
//!
//! Corre `cargo run -p 01_ownership_borrowing` desde `cheat-sheet/codigo`
//! y `cargo test -p 01_ownership_borrowing` para validar los tests.

fn main() {
    ejemplo_move();
    ejemplo_copy();
    ejemplo_clone();
    ejemplo_borrowing_inmutable();
    ejemplo_borrowing_mutable();
    ejemplo_reglas_de_exclusion();
    ejemplo_slices();
    ejemplo_no_hay_dangling_reference();
}

/// Un `String` no implementa `Copy`: asignarlo mueve el ownership.
/// Si descomentas la linea marcada, el compilador debe rechazar el programa.
fn ejemplo_move() {
    println!("--- move ---");
    let s1 = String::from("hola");
    let s2 = s1; // move: s1 ya no es utilizable

    // Descomentar la siguiente linea rompe la compilacion:
    // println!("{s1}"); // error[E0382]: borrow of moved value: `s1`

    println!("s2 = {s2}");
}

/// Los enteros son `Copy`: asignar duplica los bits, ambas variables siguen vivas.
fn ejemplo_copy() {
    println!("--- copy ---");
    let x = 5;
    let y = x; // copy, no move
    println!("x = {x}, y = {y}");
    assert_eq!(x, y);
}

/// `clone()` pide explicitamente una copia profunda del buffer del heap.
fn ejemplo_clone() {
    println!("--- clone ---");
    let s1 = String::from("hola");
    let s2 = s1.clone();
    println!("s1 = {s1}, s2 = {s2}");
    assert_eq!(s1, s2);
    // Son buffers distintos en el heap, aunque el contenido sea igual:
    assert_ne!(s1.as_ptr(), s2.as_ptr());
}

/// Una funcion que solo necesita leer puede pedir prestado con `&T`
/// en vez de tomar ownership.
fn ejemplo_borrowing_inmutable() {
    println!("--- borrowing inmutable ---");
    let s1 = String::from("hola mundo");
    let len = calcula_longitud(&s1);
    println!("La longitud de '{s1}' es {len}"); // s1 sigue siendo valida
}

fn calcula_longitud(s: &String) -> usize {
    s.len()
}

/// Una referencia mutable permite modificar el valor prestado.
fn ejemplo_borrowing_mutable() {
    println!("--- borrowing mutable ---");
    let mut s = String::from("hola");
    agrega_sufijo(&mut s);
    println!("s = {s}");
    assert_eq!(s, "hola, mundo");
}

fn agrega_sufijo(s: &mut String) {
    s.push_str(", mundo");
}

/// Demuestra la regla de exclusion: multiples `&T` conviven, pero
/// una `&mut T` no puede coexistir con otras referencias vivas.
fn ejemplo_reglas_de_exclusion() {
    println!("--- reglas de exclusion (NLL) ---");
    let mut s = String::from("hola");

    let r1 = &s;
    let r2 = &s;
    println!("r1 = {r1}, r2 = {r2}"); // ultimo uso de r1 y r2

    // Gracias a Non-Lexical Lifetimes, el prestamo mutable es valido aqui
    // porque r1 y r2 ya no se usan despues del println! anterior.
    let r3 = &mut s;
    r3.push_str("!");
    println!("r3 = {r3}");

    // Si quisieras usar r1 despues de crear r3, el compilador lo rechazaria:
    // println!("{r1}"); // error[E0502]: cannot borrow `s` as mutable because
    //                    // it is also borrowed as immutable
}

/// Un slice `&str` es una referencia a una parte contigua de un String,
/// sin tomar ownership de la coleccion completa.
fn ejemplo_slices() {
    println!("--- slices ---");
    let s = String::from("hola mundo");

    let hola: &str = &s[0..4];
    let mundo: &str = &s[5..10];
    println!("hola = '{hola}', mundo = '{mundo}'");

    assert_eq!(primera_palabra("hola mundo"), "hola");
    assert_eq!(primera_palabra("solo_una_palabra"), "solo_una_palabra");
}

/// Recibir `&str` en vez de `&String` es idiomatico: acepta ambos gracias
/// a deref coercion, sin obligar al llamador a tener un `String` propio.
fn primera_palabra(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}

/// El compilador impide devolver una referencia a un dato que se libera
/// al terminar la funcion (dangling reference). La solucion es mover el
/// ownership hacia afuera devolviendo el valor, no una referencia a el.
fn ejemplo_no_hay_dangling_reference() {
    println!("--- sin dangling references ---");
    let s = crea_string_con_ownership();
    println!("s = {s}");

    // La siguiente funcion NO compilaria si existiera:
    //
    // fn referencia_colgante() -> &String {
    //     let s = String::from("hola");
    //     &s // error[E0106]: missing lifetime specifier / valor liberado al salir de scope
    // }
}

fn crea_string_con_ownership() -> String {
    let s = String::from("hola, ownership movido hacia afuera");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_invalida_el_original_pero_el_valor_sigue_vivo() {
        let s1 = String::from("hola");
        let s2 = s1;
        assert_eq!(s2, "hola");
    }

    #[test]
    fn copy_deja_ambas_variables_utilizables() {
        let x = 10;
        let y = x;
        assert_eq!(x, y);
    }

    #[test]
    fn clone_produce_buffers_independientes() {
        let s1 = String::from("hola");
        let s2 = s1.clone();
        assert_eq!(s1, s2);
        assert_ne!(s1.as_ptr(), s2.as_ptr());
    }

    #[test]
    fn borrow_inmutable_no_consume_el_valor() {
        let s = String::from("hola mundo");
        assert_eq!(calcula_longitud(&s), 10);
        // s sigue siendo valida aqui
        assert_eq!(s, "hola mundo");
    }

    #[test]
    fn borrow_mutable_modifica_a_traves_de_la_referencia() {
        let mut s = String::from("hola");
        agrega_sufijo(&mut s);
        assert_eq!(s, "hola, mundo");
    }

    #[test]
    fn primera_palabra_encuentra_el_espacio() {
        assert_eq!(primera_palabra("hola mundo"), "hola");
    }

    #[test]
    fn primera_palabra_sin_espacios_devuelve_todo() {
        assert_eq!(primera_palabra("hola"), "hola");
    }
}
