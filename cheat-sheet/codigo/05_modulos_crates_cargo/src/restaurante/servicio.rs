//! Submodulo `restaurante::servicio`.

/// Privado al modulo `servicio`: `cocina` no lleva `pub`, asi que solo es
/// visible dentro de este archivo y sus descendientes.
mod cocina {
    pub(super) fn preparar_pedido(plato: &str) -> String {
        format!("preparando {plato}")
    }
}

pub fn tomar_pedido(plato: &str) -> String {
    // `super::cocina` no compilaria: cocina es privado a este modulo;
    // se llama por ruta relativa dentro del mismo archivo.
    cocina::preparar_pedido(plato)
}
