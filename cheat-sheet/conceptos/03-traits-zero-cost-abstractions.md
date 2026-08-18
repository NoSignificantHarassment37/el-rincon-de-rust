# 3. Traits y Zero-Cost Abstractions

Código de ejemplo: [`codigo/03_traits_zero_cost`](../codigo/03_traits_zero_cost/src/main.rs)

## 3.1 ¿Qué es un trait?

Un `trait` define un conjunto de comportamiento que un tipo puede implementar: es el equivalente de Rust a una interfaz (Java, Go) o a una type class (Haskell). A diferencia de la herencia de clases, un trait no impone una jerarquía ni un layout de datos común — solo exige que ciertos métodos existan.

```rust
trait Resumible {
    fn resumen(&self) -> String;

    // Un metodo puede tener implementacion por defecto:
    fn resumen_largo(&self) -> String {
        format!("(leer mas...) {}", self.resumen())
    }
}

struct Articulo {
    titular: String,
    contenido: String,
}

impl Resumible for Articulo {
    fn resumen(&self) -> String {
        format!("{}: {}...", self.titular, &self.contenido[..20.min(self.contenido.len())])
    }
}
```

Un detalle importante de diseño: en Rust puedes implementar un trait para un tipo **externo** (que no definiste tú), siempre que el trait o el tipo sean tuyos (la "regla huérfana" u *orphan rule*). Esto evita conflictos de implementaciones ambiguas entre crates distintos, algo que sí puede pasar con el *monkey-patching* de otros lenguajes.

## 3.2 Traits como parámetros: `impl Trait` y trait bounds

```rust
// Azucar sintactico: acepta cualquier tipo que implemente Resumible
fn notificar(item: &impl Resumible) {
    println!("¡Última hora! {}", item.resumen());
}

// Forma equivalente, mas explicita (trait bound):
fn notificar<T: Resumible>(item: &T) {
    println!("¡Última hora! {}", item.resumen());
}

// Multiples bounds con `+`:
fn notificar_debug<T: Resumible + std::fmt::Debug>(item: &T) { /* ... */ }

// where clause, mas legible con muchos bounds:
fn procesar<T, U>(t: &T, u: &U)
where
    T: Resumible,
    U: Clone + std::fmt::Debug,
{
    // ...
}
```

Estas tres formas son estrictamente equivalentes; la sintaxis `impl Trait` es azúcar para el caso simple, y `where` mejora la legibilidad cuando hay varios bounds.

## 3.3 Static dispatch vs dynamic dispatch

Aquí es donde traits se conecta directamente con *zero-cost abstractions*. Rust ofrece dos formas de polimorfismo con traits, y la diferencia es fundamental:

### Static dispatch (`impl Trait` / genéricos `<T: Trait>`)

```rust
fn notificar(item: &impl Resumible) {
    println!("{}", item.resumen());
}
```

El compilador genera una copia especializada de la función **para cada tipo concreto** con el que se llama — esto se llama *monomorphization*. Si llamas a `notificar` con un `Articulo` y con un `Tweet`, el binario final contiene dos versiones de la función, cada una con la llamada a `resumen()` resuelta e insertada directamente (posiblemente *inlined*). No hay indirección, no hay búsqueda en una tabla de punteros a función en tiempo de ejecución: el costo es idéntico a haber escrito manualmente dos funciones separadas, `notificar_articulo` y `notificar_tweet`.

Esto es exactamente lo que significa **zero-cost abstraction**, la frase que acuñó Bjarne Stroustrup y que Rust adopta como principio de diseño: *"lo que no usas, no lo pagas; y lo que usas, no podrías haberlo escrito mejor a mano"*. La abstracción (un trait genérico) desaparece por completo en tiempo de ejecución — el costo se paga solo en tiempo de compilación (más tiempo de compilación, binario más grande por la duplicación de código).

### Dynamic dispatch (`dyn Trait`)

```rust
fn notificar(item: &dyn Resumible) {
    println!("{}", item.resumen());
}

let items: Vec<Box<dyn Resumible>> = vec![
    Box::new(Articulo { /* ... */ }),
    Box::new(Tweet { /* ... */ }),
];
```

Aquí, en cambio, se usa un **trait object**: `&dyn Resumible` o `Box<dyn Resumible>` son punteros "gordos" (*fat pointers*) que contienen un puntero a los datos y un puntero a una *vtable* (tabla de funciones virtuales) generada en tiempo de compilación. Llamar a `item.resumen()` implica una indirección extra: consultar la vtable en tiempo de ejecución para saber qué implementación concreta invocar. Esto **sí** tiene un costo (pequeño, pero real) frente al static dispatch, y por eso no es "zero-cost" — es el precio que pagas a cambio de poder guardar tipos distintos que implementan el mismo trait en una sola colección homogénea, algo que el static dispatch no permite porque cada monomorphization es un tipo de función distinto.

### ¿Cuándo usar cada uno?

| | Static dispatch (`impl Trait` / genéricos) | Dynamic dispatch (`dyn Trait`) |
|---|---|---|
| Costo en runtime | Ninguno (inlined, resuelto en compilación) | Una indirección por llamada (vtable) |
| Tamaño del binario | Mayor (una copia por tipo concreto) | Menor (una sola función) |
| ¿Tipos heterogéneos en una colección? | No | Sí (`Vec<Box<dyn Trait>>`) |
| Uso típico | Rutas críticas de rendimiento, la mayoría del código | Plugins, colecciones heterogéneas, reducir tamaño de binario |

## 3.4 Traits comunes de la librería estándar

- `Debug`: permite `{:?}` en `println!`, casi siempre derivado con `#[derive(Debug)]`.
- `Clone` / `Copy`: duplicación explícita/implícita (ver capítulo 1).
- `PartialEq` / `Eq`: comparación con `==`.
- `PartialOrd` / `Ord`: comparación con `<`, `>`, y soporte para `.sort()`.
- `Default`: valores por defecto, `T::default()`.
- `From` / `Into`: conversiones entre tipos, la base de `?` para propagar errores de distinto tipo.
- `Iterator`: el trait detrás de todo el ecosistema de iteradores (ver 3.6).

La mayoría se pueden derivar automáticamente:

```rust
#[derive(Debug, Clone, PartialEq, Default)]
struct Punto { x: i32, y: i32 }
```

## 3.5 Genéricos: abstracción sin costo sobre tipos

```rust
fn el_mayor<T: PartialOrd + Copy>(lista: &[T]) -> T {
    let mut mayor = lista[0];
    for &item in lista {
        if item > mayor {
            mayor = item;
        }
    }
    mayor
}

el_mayor(&[3, 7, 2, 9]);          // T = i32
el_mayor(&[3.1, 7.4, 2.0]);       // T = f64
```

Gracias a monomorphization, `el_mayor::<i32>` y `el_mayor::<f64>` son, en el binario final, tan eficientes como si hubieras escrito dos funciones separadas a mano. El trait bound `PartialOrd + Copy` es la única forma en que el código genérico puede usar `>` y copiar valores — el compilador verifica en tiempo de compilación que **cualquier** `T` con el que se instancie la función cumple ese contrato, así que no hace falta ninguna verificación en tiempo de ejecución.

## 3.6 Iteradores: la zero-cost abstraction por excelencia

```rust
let suma: i32 = (1..=100)
    .filter(|n| n % 2 == 0)
    .map(|n| n * n)
    .sum();
```

Esta cadena declarativa —filtrar, transformar, sumar— se compila, gracias a *inlining* agresivo y monomorphization, a un único bucle equivalente al que escribirías manualmente con un `for` imperativo y sin asignaciones intermedias. El propio libro oficial de Rust usa este ejemplo para demostrar zero-cost abstractions: el código de alto nivel no es "más lento pero más bonito"; es igual de rápido que su equivalente de bajo nivel, verificado por benchmarks del propio proyecto Rust desde sus primeras versiones.

## 3.7 Resumen mental

- Un **trait** es un contrato de comportamiento, no una jerarquía de clases.
- **Static dispatch** (genéricos, `impl Trait`) = cero costo en runtime, pero un tipo concreto por instanciación (monomorphization).
- **Dynamic dispatch** (`dyn Trait`) = pequeño costo de indirección, pero permite heterogeneidad en tiempo de ejecución.
- **Zero-cost abstraction** no significa "gratis en todo sentido": significa que el costo de la abstracción se paga en tiempo de compilación (tiempo de build, tamaño de binario) y **nunca** en tiempo de ejecución frente a la alternativa escrita a mano.
- Los iteradores y los genéricos son el ejemplo más claro de este principio funcionando en la práctica.
