//! `mini_lib`: una libreria local minima, usada como dependencia de tipo
//! `path` por `05_modulos_crates_cargo` para demostrar como un crate
//! externo (aunque sea local) expone su API publica.

pub mod geometria;
pub mod texto;

/// Re-exporta los items mas usados en la raiz del crate, para que quien
/// dependa de `mini_lib` pueda escribir `mini_lib::Rectangulo` en vez de
/// `mini_lib::geometria::Rectangulo` si asi lo prefiere.
pub use geometria::Rectangulo;
