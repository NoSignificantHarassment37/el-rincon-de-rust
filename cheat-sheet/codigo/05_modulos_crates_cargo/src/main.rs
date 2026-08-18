//! Ejemplos ejecutables de modulos, crates y cargo.
//! Ver la explicacion conceptual en `cheat-sheet/conceptos/05-modulos-crates-cargo.md`.
//!
//! Arbol de modulos de este crate binario:
//!   main.rs
//!   restaurante.rs          -> mod restaurante
//!   restaurante/hosting.rs  -> mod restaurante::hosting
//!   restaurante/servicio.rs -> mod restaurante::servicio (con `mod cocina` privado adentro)
//!
//! Ademas depende del crate local `mini_lib` (path = "../mini_lib" en
//! Cargo.toml), demostrando como se usa un crate externo.

mod restaurante;

// Trae `restaurante::hosting` al scope para no repetir la ruta completa.
use restaurante::hosting;

fn ejemplo_rutas_y_modulos() {
    println!("--- modulos: rutas absolutas y relativas ---");

    // Ruta absoluta, empieza en `crate::`:
    let msg1 = crate::restaurante::hosting::agregar_a_lista_espera("Ana");
    println!("{msg1}");

    // Ruta relativa, via `use hosting` de arriba:
    let msg2 = hosting::asignar_mesa(7);
    println!("{msg2}");

    // Via el `pub use` re-exportado en restaurante.rs:
    let msg3 = restaurante::agregar_a_lista_espera("Luis");
    println!("{msg3}");

    let msg4 = restaurante::servicio::tomar_pedido("pasta");
    println!("{msg4}");
}

/// Usa el crate externo local `mini_lib` (dependencia `path` en Cargo.toml).
/// Esto es exactamente lo mismo que usar un crate de crates.io como `rand`:
/// se declara en `[dependencies]` y se trae al scope con `use`.
fn ejemplo_crate_externo() {
    println!("--- crate externo: mini_lib ---");

    // `Rectangulo` esta re-exportado en la raiz del crate (`pub use` en lib.rs):
    let r = mini_lib::Rectangulo::new(4.0, 5.0);
    println!("area = {}, perimetro = {}", r.area(), r.perimetro());
    assert_eq!(r.area(), 20.0);

    // El modulo `texto` se accede con su ruta completa dentro de mini_lib:
    let invertido = mini_lib::texto::invertir("cargo");
    println!("invertido = {invertido}");
    assert_eq!(invertido, "ograc");

    assert!(mini_lib::texto::es_palindromo("Anita lava la tina"));
}

fn main() {
    ejemplo_rutas_y_modulos();
    ejemplo_crate_externo();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hosting_agrega_a_lista_espera() {
        assert_eq!(
            hosting::agregar_a_lista_espera("Ana"),
            "Ana fue agregado a la lista de espera"
        );
    }

    #[test]
    fn servicio_toma_pedido_usando_modulo_privado_cocina() {
        assert_eq!(
            restaurante::servicio::tomar_pedido("pasta"),
            "preparando pasta"
        );
    }

    #[test]
    fn re_export_de_restaurante_funciona() {
        assert_eq!(
            restaurante::agregar_a_lista_espera("Luis"),
            "Luis fue agregado a la lista de espera"
        );
    }

    #[test]
    fn mini_lib_geometria_funciona_como_crate_externo() {
        let r = mini_lib::Rectangulo::new(3.0, 3.0);
        assert!(r.es_cuadrado());
        assert_eq!(r.perimetro(), 12.0);
    }

    #[test]
    fn mini_lib_texto_funciona_como_crate_externo() {
        assert_eq!(mini_lib::texto::invertir("abc"), "cba");
    }
}
