# 5. Módulos, Crates y Cargo

Código de ejemplo: [`codigo/05_modulos_crates_cargo`](../codigo/05_modulos_crates_cargo/src/main.rs) (usa además la librería local [`codigo/mini_lib`](../codigo/mini_lib/src/lib.rs))

Este capítulo cubre el llamado *sistema de módulos* de Rust, que en realidad son cuatro conceptos distintos y anidados: **paquetes** (packages) → **crates** → **módulos** (modules) → **rutas** (paths). Entender la jerarquía completa es lo que hace que `use`, `pub`, y la organización de archivos dejen de sentirse arbitrarios.

## 5.1 La jerarquía completa

```
Workspace (opcional, agrupa varios packages)
└── Package (lo que describe un Cargo.toml)
    ├── (como mucho) 1 crate de librería  → src/lib.rs
    └── (0 o más) crates binarios          → src/main.rs, src/bin/*.rs
        └── cada crate tiene un árbol de módulos
            └── cada módulo puede contener items: fn, struct, enum, otro mod...
```

- **Package**: una unidad que Cargo sabe construir. Se describe con un `Cargo.toml`. Puede contener como máximo un crate de librería, y cero o más crates binarios.
- **Crate**: la unidad de compilación más pequeña que el compilador (`rustc`) considera de una vez. Puede ser una librería (`lib`) o un ejecutable (`bin`).
- **Módulo** (`mod`): organiza el código *dentro* de un crate en un espacio de nombres jerárquico, y controla qué es público y qué es privado.
- **Ruta** (`path`): cómo se nombra un item dentro del árbol de módulos, como una ruta de sistema de archivos (`crate::modulo::submodulo::funcion`).

## 5.2 Módulos: organizar código dentro de un crate

```rust
// src/lib.rs (o src/main.rs)
mod restaurante {
    pub mod hosting {
        pub fn agregar_a_lista_espera() {}

        fn asignar_mesa() {} // privado por defecto
    }

    mod cocina { // el modulo mismo es privado
        pub fn preparar_pedido() {}
    }
}
```

Por defecto, **todo es privado** en Rust: un módulo, una función, un struct o un campo solo son visibles fuera de su módulo si se marcan explícitamente con `pub`. Esto es la aplicación del mismo principio de "presunción restrictiva por defecto" que vimos con `mut` en el capítulo 4 — aquí aplicado a visibilidad en vez de a mutabilidad. Un módulo hijo, sin embargo, **sí** puede ver los items privados de sus ancestros sin necesidad de `pub`.

## 5.3 Rutas: absolutas vs relativas

```rust
pub fn eat_at_restaurant() {
    // Ruta absoluta, empieza desde la raiz del crate:
    crate::restaurante::hosting::agregar_a_lista_espera();

    // Ruta relativa, empieza desde el modulo actual:
    restaurante::hosting::agregar_a_lista_espera();
}
```

Se prefiere la ruta absoluta (`crate::...`) cuando el código que la usa probablemente se mueva de módulo con el tiempo — la ruta sigue siendo correcta sin ajustes. `super::` navega al módulo padre, análogo a `..` en un sistema de archivos.

## 5.4 `use`: traer rutas al scope

Escribir la ruta completa cada vez es tedioso. `use` crea un atajo, análogo a un symlink:

```rust
use crate::restaurante::hosting;

pub fn eat_at_restaurant() {
    hosting::agregar_a_lista_espera(); // ya no hace falta la ruta completa
}

// Renombrar para evitar colisiones de nombres:
use std::fmt::Result;
use std::io::Result as IoResult;

// Re-exportar: hace que quien use TU crate pueda acceder a esto
// como si estuviera definido directamente en tu módulo (API pública):
pub use crate::restaurante::hosting;
```

## 5.5 Un módulo, un archivo: convención moderna (Rust 2018+)

Cuando un árbol de módulos crece, se separa en archivos. La declaración `mod nombre;` (sin cuerpo, terminada en `;`) le dice al compilador: *"el contenido de este módulo está en otro archivo"*.

```
src/
├── main.rs           -> `mod restaurante;`
├── restaurante.rs     -> `pub mod hosting;` `pub mod servicio;`
└── restaurante/
    ├── hosting.rs      -> contenido del submodulo hosting
    └── servicio.rs      -> contenido del submodulo servicio
```

Esta es la convención moderna (sin `mod.rs`): un módulo `foo` con submódulos vive en `foo.rs` (el contenido de `foo` mismo) más una carpeta `foo/` (sus hijos). La convención antigua (`foo/mod.rs`) sigue funcionando pero ya no es la recomendada, porque tener muchos archivos llamados `mod.rs` abiertos en el editor es confuso.

## 5.6 Crates: binario vs librería

Un **crate binario** tiene un punto de entrada `fn main()` y produce un ejecutable. Un **crate de librería** no tiene `main`; expone una API pública para que otros crates la usen. La raíz de un crate binario es `src/main.rs`; la de uno de librería, `src/lib.rs`.

Un package puede tener ambos a la vez: una librería con la lógica, y uno o más binarios delgados que la usan (patrón muy común: la mayoría del código va en `src/lib.rs`, testeable como librería, y `src/main.rs` es solo el punto de entrada de CLI que la invoca).

```
mi_paquete/
├── Cargo.toml
├── src/
│   ├── lib.rs      -> crate de libreria `mi_paquete`
│   ├── main.rs     -> crate binario `mi_paquete` (usa la libreria con `use mi_paquete::...`)
│   └── bin/
│       └── otra_herramienta.rs  -> otro crate binario adicional
```

## 5.7 Crates externos y `Cargo.toml`

`Cargo.toml` es el manifiesto del package:

```toml
[package]
name = "mi_paquete"
version = "0.1.0"
edition = "2024"

[dependencies]
rand = "0.8.7"           # crates.io, se resuelve por SemVer
serde = { version = "1", features = ["derive"] }
mini_lib = { path = "../mini_lib" }              # crate local, mismo repo
# git_dep = { git = "https://github.com/user/repo" }

[dev-dependencies]
# solo disponibles para tests, examples y benches, no en el binario final
```

### Versionado semántico (SemVer)

`"0.8.7"` en realidad significa `^0.8.7`: acepta cualquier versión compatible, es decir `>=0.8.7, <0.9.0` (para versiones `0.x`, un cambio en el segundo número ya se considera incompatible; a partir de `1.0.0`, solo el primer número indica incompatibilidad). Cargo resuelve todas las dependencias (y las dependencias de las dependencias) a versiones concretas y las fija en `Cargo.lock` — ese archivo sí debe commitearse en un binario/aplicación (garantiza builds reproducibles) aunque convencionalmente se omite en librerías puras que se publican a crates.io.

### `[features]`: compilación condicional opcional

```toml
[features]
default = ["std"]
std = []
json = ["dep:serde_json"]
```

Las *features* permiten compilar partes opcionales de una librería (por ejemplo, soporte para `no_std`, o integración opcional con otra librería) sin forzar esa dependencia a todos los usuarios.

## 5.8 Workspaces: varios packages, un solo `Cargo.lock`

```toml
# Cargo.toml en la raiz del workspace
[workspace]
resolver = "2"
members = [
    "core",
    "cli",
    "web",
]
```

Un workspace agrupa varios packages que comparten un mismo `target/` de compilación y un mismo `Cargo.lock` — todas las dependencias compartidas se compilan una sola vez, y `cargo build`/`cargo test` en la raíz opera sobre todos los miembros. Es el patrón natural para dividir un proyecto grande en piezas independientes que aun así se versionan y compilan juntas. **El propio directorio `cheat-sheet/codigo` de este repositorio es un workspace** con 6 miembros: uno por cada tema, más `mini_lib`.

## 5.9 Comandos esenciales de Cargo

| Comando | Qué hace |
|---|---|
| `cargo new nombre` | Crea un package nuevo (con git init incluido) |
| `cargo init` | Igual, pero en un directorio ya existente |
| `cargo build` | Compila en modo debug (rápido de compilar, sin optimizar) |
| `cargo build --release` | Compila con optimizaciones agresivas (lento de compilar, rápido de ejecutar) |
| `cargo run` | Compila (si hace falta) y ejecuta el binario |
| `cargo check` | Verifica que compile, sin generar el binario final — mucho más rápido, ideal en desarrollo iterativo |
| `cargo test` | Compila y corre todos los tests (`#[test]`, doctests, tests de integración) |
| `cargo doc --open` | Genera documentación HTML a partir de comentarios `///` y la abre en el navegador |
| `cargo fmt` | Formatea el código según el estilo estándar |
| `cargo clippy` | Linter que detecta patrones no idiomáticos o potencialmente erróneos |
| `cargo add nombre` | Agrega una dependencia a `Cargo.toml` (resuelve la última versión compatible) |
| `cargo tree` | Muestra el árbol completo de dependencias resueltas |
| `cargo publish` | Publica el crate en crates.io |

En este repo, como `codigo/` es un workspace, cada comando puede apuntar a un miembro específico con `-p <nombre>` (por ejemplo, `cargo test -p mini_lib`) o correr sobre todos los miembros si se omite `-p`.

## 5.10 Resumen mental

- **Package** = lo que Cargo.toml describe. **Crate** = lo que rustc compila de una vez (lib o bin). **Módulo** = cómo organizas código *dentro* de un crate. **Ruta** = cómo nombras un item dentro del árbol de módulos.
- Todo es privado por defecto; `pub` es un permiso explícito, igual que `mut` lo es para mutabilidad.
- `use` es un atajo de ruta, no una copia de código — no tiene costo en tiempo de ejecución.
- `Cargo.lock` fija versiones exactas para builds reproducibles; `Cargo.toml` expresa rangos aceptables vía SemVer.
- Un workspace comparte `Cargo.lock` y `target/` entre varios packages relacionados.
