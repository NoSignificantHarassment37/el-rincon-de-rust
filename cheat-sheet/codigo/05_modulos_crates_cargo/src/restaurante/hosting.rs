//! Submodulo `restaurante::hosting`.

pub fn agregar_a_lista_espera(nombre: &str) -> String {
    format!("{nombre} fue agregado a la lista de espera")
}

pub fn asignar_mesa(numero: u32) -> String {
    format!("mesa {numero} asignada")
}
