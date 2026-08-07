# CHEAT SHEET

## Comandos

`cargo new <nombre del proyecto>` inicializa en el directorio actual, un nuevo proyecto rust
`cargo run` ejecuta todo el proceso de build del ejecutable y arranca el ejecutable
`cargo build` solo ejecuta el proceso de build del ejecutable con target de desarrollo
`cargo build --release` solo ejecuta el proceso de build del ejecutable con target de distribucion, lo que hace que el compilador optimice agresivamente.
`cargo check` sirve para detectar errores de compilacion, pero sin compilar el programa
`cargo doc --open`

## Dependencias

Un metodo para agregar una dependencia al proyecto, es modificar manualmente cargo.toml, debajo de la seccion [dependencies] con la sintaxis:
`<nombre del paquete en crates.io> = "version"`

## Shadowing

El compilador permite declarar variables con el mismo nombre dentro del mismo scope, lo que provoca que la nueva sea la referencia, la anterior sigue existiendo en memoria, pero ya no se puede alcanzar desde rust.

## mutabilidad vs inmutabilidad

// valido
let mut x: i32 = 1;
x = 32;

// invalido
let x: i32 = 1;
x = 32;