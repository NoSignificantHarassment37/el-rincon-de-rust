# 1. Ownership, Borrowing y Referencias

Código de ejemplo: [`codigo/01_ownership_borrowing`](../codigo/01_ownership_borrowing/src/main.rs)

## 1.1 El problema que ownership resuelve

Los lenguajes de sistemas tienen que decidir cómo gestionan la memoria dinámica (heap):

- **Garbage collection** (Go, Java, Python): un proceso en segundo plano rastrea qué está en uso y libera lo que ya nadie referencia. Es seguro, pero cuesta tiempo de CPU y produce pausas impredecibles.
- **Gestión manual** (C, C++): el programador llama a `malloc`/`free` (o `new`/`delete`). Es rápido, pero un error humano produce *use-after-free*, *double-free* o fugas de memoria — bugs de seguridad reales, no solo de rendimiento.

Rust elige una tercera vía: el compilador demuestra, en tiempo de compilación, que la memoria se libera exactamente una vez y que nunca se usa después de liberada. No hay recolector de basura ni costo en tiempo de ejecución: las reglas se verifican estáticamente mediante el **ownership** (propiedad) y el **borrow checker**. Esto es parte de lo que Rust llama *zero-cost abstractions* (ver [`03-traits-zero-cost-abstractions.md`](03-traits-zero-cost-abstractions.md)): pagas la seguridad en tiempo de compilación, no en tiempo de ejecución.

## 1.2 Las reglas de ownership

1. Cada valor en Rust tiene una variable que es su **dueña** (*owner*).
2. Solo puede haber **un dueño** a la vez.
3. Cuando el dueño sale de *scope*, el valor se libera automáticamente (se llama a `drop`).

Estas tres reglas, combinadas con el sistema de tipos, son suficientes para eliminar toda una clase de bugs de memoria sin necesidad de un recolector de basura.

```rust
{
    let s = String::from("hola"); // s es dueña de los bytes "hola" en el heap
    // s es válida aquí
} // el scope termina, Rust llama a `drop(s)` automáticamente, la memoria se libera
```

Este patrón —liberar un recurso cuando la variable que lo posee sale de scope— se llama **RAII** (*Resource Acquisition Is Initialization*), un concepto que Rust hereda de C++ pero que aplica también a archivos, sockets, locks, etc. No es exclusivo de la memoria: cualquier tipo que implemente el trait `Drop` puede definir su propia lógica de limpieza.

## 1.3 Move: por qué asignar no siempre copia

```rust
let s1 = String::from("hola");
let s2 = s1; // "move": s1 se invalida, s2 es ahora la dueña
// println!("{s1}"); // ERROR de compilación: value borrowed after move
```

`String` es un tipo que gestiona memoria en el heap: internamente es un puntero, una longitud y una capacidad (24 bytes en un sistema de 64 bits). Si `let s2 = s1` copiara solo esos 24 bytes de metadata (una "shallow copy"), tendríamos **dos** variables apuntando al mismo buffer del heap. Cuando ambas salieran de scope, Rust llamaría a `drop` dos veces sobre el mismo puntero: un **double-free**, uno de los bugs de memoria más peligrosos que existen.

Para evitarlo, Rust no copia superficialmente: **invalida** `s1`. A esto se le llama *move* (mover). Después del move, `s1` deja de ser válida y el compilador rechaza cualquier uso posterior de `s1` con un error en tiempo de compilación, no en tiempo de ejecución. Conceptualmente es una "shallow copy + invalidación del original", así que no hay costo de rendimiento: mover un `String` es tan barato como copiar 24 bytes, sin importar cuántos datos haya en el heap.

Esto también aplica al pasar valores a funciones y al retornarlos:

```rust
fn toma_ownership(s: String) {
    println!("{s}");
} // s sale de scope aquí, se libera la memoria

let s = String::from("hola");
toma_ownership(s);
// s ya no es válida: su ownership se movió dentro de la función
```

## 1.4 El trait `Copy`: cuando mover sí es copiar

Los tipos simples que viven enteramente en el stack (enteros, `f64`, `bool`, `char`, tuplas de tipos `Copy`) implementan el trait `Copy`. Para estos tipos, "mover" y "copiar" son la misma operación bit a bit, así que Rust simplemente los copia y **no** invalida el original:

```rust
let x = 5;
let y = x; // se copia, no se mueve
println!("{x} {y}"); // válido: ambas variables son utilizables
```

Un tipo no puede implementar `Copy` si implementa `Drop` (o si contiene un campo que no es `Copy`), porque la semántica sería contradictoria: `Copy` asume que duplicar los bytes es seguro y suficiente, mientras que `Drop` asume que hay un recurso externo (heap, archivo, socket) que debe liberarse de forma controlada y única.

## 1.5 Clone: duplicar explícitamente

Cuando sí queremos dos copias independientes de datos en el heap, lo pedimos explícitamente con `.clone()`:

```rust
let s1 = String::from("hola");
let s2 = s1.clone(); // copia profunda: nuevo buffer en el heap
println!("{s1} {s2}"); // ambas son válidas
```

La distinción entre "mover por defecto" y "clonar explícitamente" es una decisión de diseño deliberada: en Rust, cualquier operación potencialmente costosa (como duplicar un buffer de heap) es **visible en el código**. Nunca hay una copia profunda oculta detrás de un simple `=`.

## 1.6 Borrowing: usar sin tomar ownership

Mover el ownership a cada función sería muy incómodo: tendríamos que devolver el valor en cada retorno solo para poder seguir usándolo. Rust ofrece **referencias** (`&T`) para *pedir prestado* (borrow) un valor sin tomar su ownership:

```rust
fn calcula_longitud(s: &String) -> usize {
    s.len()
} // s (la referencia) sale de scope, pero como no es dueña, no se libera nada

let s1 = String::from("hola");
let len = calcula_longitud(&s1);
println!("La longitud de '{s1}' es {len}."); // s1 sigue siendo válida
```

`&s1` crea una referencia que **apunta** a `s1` sin tomar su ownership. Cuando la referencia sale de scope, no pasa nada especial porque no es dueña de los datos. Este mecanismo se llama **borrowing**.

## 1.7 Referencias mutables y las reglas de exclusión

Por defecto, una referencia es inmutable: no se puede modificar el valor prestado a través de ella. Para modificar, se necesita una referencia mutable, `&mut T`:

```rust
fn agrega_sufijo(s: &mut String) {
    s.push_str(", mundo");
}

let mut s = String::from("hola");
agrega_sufijo(&mut s);
println!("{s}"); // "hola, mundo"
```

Aquí está la regla más importante del borrow checker, la que hace que Rust sea *seguro en concurrencia por diseño* (ver [`04-mutabilidad-concurrencia.md`](04-mutabilidad-concurrencia.md)):

> En un scope dado, para un mismo dato puedes tener **o bien** una referencia mutable (`&mut T`), **o bien** cualquier número de referencias inmutables (`&T`), pero **nunca ambas a la vez**.

```rust
let mut s = String::from("hola");

let r1 = &s;
let r2 = &s;
println!("{r1} y {r2}"); // OK: múltiples referencias inmutables

let r3 = &mut s; // OK, siempre que r1 y r2 ya no se usen después de este punto
r3.push_str("!");
```

¿Por qué esta regla? Porque previene **data races** en tiempo de compilación: una data race ocurre cuando dos punteros acceden al mismo dato al mismo tiempo, al menos uno de ellos escribe, y no hay sincronización. Si el compilador garantiza que mientras existe una referencia mutable no puede existir ninguna otra referencia (ni mutable ni inmutable), es imposible que dos "lectores/escritores" pisen el mismo dato de forma insegura — sin necesidad de locks ni de un runtime que lo vigile.

Desde la edición 2021, Rust usa **Non-Lexical Lifetimes (NLL)**: el scope de una referencia termina en su último uso real, no al final del bloque léxico. Esto permite código como:

```rust
let mut s = String::from("hola");
let r1 = &s;
println!("{r1}"); // último uso de r1
let r2 = &mut s; // OK: r1 ya "murió" en el punto anterior
r2.push_str("!");
```

## 1.8 Referencias colgantes (dangling references)

En C, es fácil devolver un puntero a memoria que ya fue liberada. En Rust, el compilador lo impide directamente:

```rust
fn referencia_colgante() -> &String { //         ERROR: falta lifetime,
    let s = String::from("hola");     //         y aunque lo tuviera, s
    &s                                //         se libera al salir de scope
} // s sale de scope y se libera aquí; &s apuntaría a memoria inválida
```

El compilador rechaza este código porque `s` es local a la función: al terminar la función, `s` se libera y la referencia devuelta apuntaría a memoria ya liberada. La solución natural es devolver `s` directamente (mover el ownership hacia afuera) en lugar de una referencia:

```rust
fn no_hay_problema() -> String {
    let s = String::from("hola");
    s // se mueve el ownership al llamador
}
```

Este análisis de "cuánto tiempo vive una referencia frente a cuánto tiempo vive el dato al que apunta" es precisamente el trabajo de los **lifetimes**, el tema del siguiente documento: [`02-lifetimes-pattern-matching.md`](02-lifetimes-pattern-matching.md).

## 1.9 Slices: referencias a una parte de una colección

Un *slice* es una referencia a una secuencia contigua de elementos, sin tomar ownership de la colección completa. `&str` (una cadena de texto) es, de hecho, un slice de un `String` o de un literal:

```rust
let s = String::from("hola mundo");

let hola: &str = &s[0..4];   // slice de los primeros 4 bytes
let mundo: &str = &s[5..10]; // slice de "mundo"

fn primera_palabra(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}
```

Que `primera_palabra` reciba `&str` en vez de `&String` es una convención idiomática: `&str` acepta tanto un `&String` (por *deref coercion*) como un literal `"texto"`, así que la función es más flexible sin perder seguridad.

## 1.10 Resumen mental

| Operación | ¿Quién es dueño después? | Costo |
|---|---|---|
| `let s2 = s1;` (tipo no-`Copy`) | `s2`; `s1` queda inválida (move) | O(1), copia de metadata |
| `let s2 = s1;` (tipo `Copy`) | Ambos son válidos e independientes | O(1), copia de bits |
| `let s2 = s1.clone();` | Ambos son válidos e independientes | O(n), copia profunda explícita |
| `fn f(s: &T)` | El llamador sigue siendo dueño | O(1), solo un puntero |
| `fn f(s: &mut T)` | El llamador sigue siendo dueño, pero no puede tener otras referencias mientras dure el préstamo | O(1), solo un puntero |

**La idea central:** en Rust, la pregunta "¿quién es responsable de liberar esta memoria?" siempre tiene una respuesta única y verificada en tiempo de compilación. Eso es lo que hace posible tener seguridad de memoria sin garbage collector.
