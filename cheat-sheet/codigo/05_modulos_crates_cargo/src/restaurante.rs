//! Modulo `restaurante`. La declaracion `pub mod hosting;` (sin cuerpo,
//! terminada en `;`) le dice al compilador que busque el contenido en
//! `restaurante/hosting.rs` -- convencion moderna sin `mod.rs`.

pub mod hosting;
pub mod servicio;

/// Re-exporta `hosting` para que el llamador pueda usar
/// `restaurante::agregar_a_lista_espera` directamente si prefiere una
/// API mas plana (mismo mecanismo de `pub use` que usa `mini_lib`).
pub use hosting::agregar_a_lista_espera;
