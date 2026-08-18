# 2. Lifetimes, Tipos Algebraicos y Pattern Matching

Código de ejemplo: [`codigo/02_lifetimes_pattern_matching`](../codigo/02_lifetimes_pattern_matching/src/main.rs)

Este documento junta dos temas que a primera vista parecen distintos, pero que en Rust están profundamente conectados: los **lifetimes** (que garantizan que las referencias del capítulo anterior nunca apunten a memoria inválida) y los **tipos algebraicos con pattern matching** (que permiten modelar datos y, sobre todo, obligan a manejar todos los casos posibles).

## Parte A — Lifetimes

### 2.1 ¿Qué problema resuelven los lifetimes?

En [`01-ownership-borrowing-referencias.md`](01-ownership-borrowing-referencias.md) vimos que el compilador rechaza referencias colgantes (*dangling references*). El mecanismo que usa para detectarlas en casos no triviales es el análisis de **lifetimes**: cada referencia tiene una región del código en la que es válida, y el compilador verifica que ninguna referencia se use fuera de la región en la que los datos a los que apunta siguen vivos.

La mayoría de las veces el compilador infiere los lifetimes automáticamente (*lifetime elision*), por eso raramente los escribes explícitamente en código simple. Pero cuando una función recibe **varias** referencias y devuelve una referencia, el compilador no siempre puede adivinar de cuál de las entradas depende la salida — ahí es donde se vuelven explícitos.

```rust
fn mas_largo(x: &str, y: &str) -> &str { // ERROR: falta especificar el lifetime
    if x.len() > y.len() { x } else { y }
}
```

Este código no compila. El compilador no sabe si la referencia devuelta vivirá tanto como `x`, como `y`, o como ninguno de los dos — y esa información es necesaria para verificar, en el sitio de la llamada, que el resultado no se use después de que el dato original se libere.

### 2.2 Anotaciones de lifetime: un contrato, no un cambio de duración

```rust
fn mas_largo<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

`'a` es un parámetro genérico (como `T` para tipos), pero para lifetimes. Esta firma dice: *"devuelvo una referencia que vive, como mínimo, tanto como la más corta de `x` e `y`"*. Es importante entender que **`'a` no cambia cuánto viven `x` o `y`**; solo describe una relación que el compilador puede usar para verificar el código que llama a esta función. Es un contrato entre la función y sus llamadores, no una instrucción que altere el tiempo de vida real de nada.

```rust
let s1 = String::from("larga cadena");
let resultado;
{
    let s2 = String::from("corta");
    resultado = mas_largo(s1.as_str(), s2.as_str());
    println!("{resultado}"); // OK: s2 todavía vive aquí
}
// println!("{resultado}"); // ERROR aquí: s2 ya no vive, y resultado podría depender de s2
```

El compilador rechaza el segundo `println!` porque, según la firma `'a`, `resultado` podría estar apuntando a `s2` — y `s2` ya no existe fuera del bloque interno.

### 2.3 Reglas de elision (por qué normalmente no escribes lifetimes)

El compilador aplica tres reglas automáticas antes de exigir anotaciones explícitas:

1. Cada parámetro de referencia recibe su propio lifetime.
2. Si hay exactamente un parámetro de entrada con referencia, ese lifetime se asigna a todas las referencias de salida.
3. Si uno de los parámetros es `&self` o `&mut self` (métodos), el lifetime de `self` se asigna a las referencias de salida.

Por eso `fn primera_palabra(s: &str) -> &str` (del capítulo anterior) compila sin anotaciones: cae en la regla 2. Pero `mas_largo` tiene dos parámetros de referencia y ninguno es `self`, así que ninguna regla aplica y hay que anotar manualmente.

### 2.4 Lifetimes en structs

Un `struct` puede guardar una referencia, pero entonces necesita un parámetro de lifetime que garantice que la instancia del struct no puede sobrevivir a los datos que referencia:

```rust
struct Extracto<'a> {
    parte: &'a str,
}

let novela = String::from("Llámame Ismael. Hace algunos años...");
let primera_oracion = novela.split('.').next().unwrap();
let extracto = Extracto { parte: primera_oracion };
// `extracto` no puede usarse después de que `novela` se libere.
```

### 2.5 El lifetime estático `'static`

`'static` significa que la referencia vive durante todo el programa. Los literales de cadena (`&str`) son `'static` porque están incrustados directamente en el binario compilado:

```rust
let s: &'static str = "esta cadena vive para siempre";
```

Se debe usar con cuidado: forzar `'static` en una firma genérica (`T: 'static`) es una restricción fuerte que a menudo se pide por error cuando el problema real es otro (frecuentemente relacionado con *ownership* en hilos, ver [`04-mutabilidad-concurrencia.md`](04-mutabilidad-concurrencia.md)).

---

## Parte B — Tipos algebraicos y pattern matching

### 2.6 ¿Qué es un tipo algebraico?

Un **tipo algebraico de datos** (ADT) es un tipo compuesto construido a partir de otros tipos usando dos operaciones básicas:

- **Producto** (`AND`): tienes un valor de tipo `A` **y** uno de tipo `B` **y**... — esto es un `struct`.
- **Suma** (`OR`): tienes un valor que es **o bien** de tipo `A`, **o bien** de tipo `B`, **o bien**... — esto es un `enum`.

Rust tiene ambos de forma nativa y de primera clase, lo cual lo distingue de lenguajes como Java o Python donde los "tipos suma" hay que simularlos con herencia, `Optional`, o convenciones.

### 2.7 Structs: tipos producto

```rust
struct Usuario {
    nombre: String,
    edad: u8,
    activo: bool,
}
```

Un `Usuario` siempre tiene los tres campos presentes simultáneamente. El número de estados posibles es el producto cartesiano de los estados de cada campo — de ahí "tipo producto".

### 2.8 Enums: tipos suma

```rust
enum Forma {
    Circulo { radio: f64 },
    Rectangulo { ancho: f64, alto: f64 },
    Triangulo { base: f64, altura: f64 },
}
```

Un valor de tipo `Forma` es **exactamente uno** de esos tres casos, nunca una mezcla, y cada variante puede llevar sus propios datos asociados. Esto es mucho más expresivo que, por ejemplo, un struct con un campo `tipo: String` y varios campos opcionales que "a veces aplican" — un patrón común en lenguajes sin tipos suma, y una fuente constante de estados inválidos representables.

### 2.9 `Option<T>`: el fin de `null`

Rust no tiene `null`. En su lugar, la librería estándar define:

```rust
enum Option<T> {
    Some(T),
    None,
}
```

Cualquier valor que "podría no estar presente" se modela como `Option<T>`, nunca como un puntero que podría ser nulo. La ventaja no es solo estilística: el compilador **obliga** a manejar el caso `None` antes de poder usar el valor interior. Es imposible olvidarlo y producir el equivalente a un *null pointer dereference* — la clase de bug que Tony Hoare, quien inventó `null`, llamó su "error de mil millones de dólares".

### 2.10 `Result<T, E>`: errores como valores, no como excepciones

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

En vez de lanzar excepciones que pueden propagarse silenciosamente por rutas no evidentes en la firma de una función, las funciones que pueden fallar devuelven `Result<T, E>`. El tipo de retorno documenta, de forma verificada por el compilador, que la operación puede fallar — y obliga a decidir explícitamente qué hacer en cada caso.

### 2.11 `match`: pattern matching exhaustivo

```rust
fn describe(forma: &Forma) -> String {
    match forma {
        Forma::Circulo { radio } => format!("círculo de radio {radio}"),
        Forma::Rectangulo { ancho, alto } => format!("rectángulo {ancho}x{alto}"),
        Forma::Triangulo { base, altura } => format!("triángulo base {base} altura {altura}"),
    }
}
```

`match` no es un `switch` glorificado: el compilador verifica **exhaustividad**. Si `Forma` ganara una cuarta variante `Poligono` mañana, este `match` dejaría de compilar hasta que se agregue el caso correspondiente. Esto convierte "olvidé manejar un caso nuevo" —un bug clásico al extender un tipo— en un error de compilación en lugar de un bug silencioso en producción.

Patrones útiles:

```rust
// Extraer un valor de un Option, con un default:
let x: Option<i32> = None;
let valor = match x {
    Some(v) => v,
    None => 0,
};

// El mismo patrón, más idiomático:
let valor = x.unwrap_or(0);

// Guard conditions:
let n = 4;
match n {
    x if x % 2 == 0 => println!("{x} es par"),
    x => println!("{x} es impar"),
}

// Bindings con @:
match n {
    edad @ 0..=17 => println!("{edad}: menor de edad"),
    edad @ 18..=64 => println!("{edad}: adulto"),
    edad => println!("{edad}: adulto mayor"),
}

// Desestructurar tuplas y structs directamente:
let (a, b) = (1, 2);
let Usuario { nombre, edad, .. } = usuario; // `..` ignora el resto de campos
```

### 2.12 `if let` y `while let`: pattern matching abreviado

Cuando solo te importa **un** patrón y quieres ignorar el resto, `match` con un brazo `_ => {}` es verboso. `if let` es el azúcar sintáctico para ese caso:

```rust
let config_max: Option<u8> = Some(3);

// Verboso:
match config_max {
    Some(max) => println!("máximo configurado: {max}"),
    _ => (),
}

// Idiomático:
if let Some(max) = config_max {
    println!("máximo configurado: {max}");
}
```

`while let` repite el patrón mientras siga matcheando — típicamente para vaciar una pila o una cola:

```rust
let mut pila = vec![1, 2, 3];
while let Some(top) = pila.pop() {
    println!("{top}");
}
```

### 2.13 Por qué esto importa junto con ownership

Cuando haces `match` sobre un valor que no es `Copy`, cada brazo decide si **mueve** el contenido, lo **referencia** o lo **modifica**, siguiendo exactamente las mismas reglas del capítulo 1:

```rust
let opcional = Some(String::from("hola"));

match opcional {
    Some(ref s) => println!("prestado: {s}"), // &String, opcional sigue vivo
    None => {}
}

match opcional {
    Some(s) => println!("movido: {s}"), // String, opcional se consume
    None => {}
}
```

Esta es la razón por la que "tipos algebraicos + pattern matching" y "ownership + borrowing" comparten capítulo conceptualmente: `match` es, en esencia, el punto donde el compilador decide —campo por campo, variante por variante— si algo se mueve, se copia o se presta.

## 2.14 Resumen mental

| Concepto | Pregunta que responde |
|---|---|
| Lifetime (`'a`) | "¿Hasta cuándo es válida esta referencia, en relación a otras?" |
| `struct` (producto) | "¿Qué datos coexisten siempre juntos?" |
| `enum` (suma) | "¿Cuáles son *todos* los estados posibles, mutuamente excluyentes?" |
| `Option<T>` | "¿Puede este valor estar ausente?" (sin `null`) |
| `Result<T, E>` | "¿Puede esta operación fallar, y con qué error?" |
| `match` exhaustivo | "¿Manejé *todos* los casos posibles?" (verificado en compilación) |
