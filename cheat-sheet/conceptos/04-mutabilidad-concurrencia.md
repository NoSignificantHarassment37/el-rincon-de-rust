# 4. Mutabilidad Explícita y Concurrencia

Código de ejemplo: [`codigo/04_mutabilidad_concurrencia`](../codigo/04_mutabilidad_concurrencia/src/main.rs)

Este capítulo conecta directamente con las reglas de exclusión de referencias del capítulo 1 (["o una `&mut T`, o varias `&T`, nunca ambas"](01-ownership-borrowing-referencias.md#17-referencias-mutables-y-las-reglas-de-exclusión)): esa misma regla, aplicada entre hilos en vez de entre líneas de código, es la razón por la que Rust puede prevenir *data races* en programas concurrentes sin necesidad de un recolector de basura ni de un runtime que vigile en segundo plano.

## 4.1 Mutabilidad explícita: `let` vs `let mut`

En Rust, **todo es inmutable por defecto**. Para poder reasignar o modificar un valor, hay que declararlo explícitamente con `mut`:

```rust
let x = 5;
// x = 6; // ERROR: cannot assign twice to immutable variable

let mut y = 5;
y = 6; // OK
```

Esto es una decisión de diseño deliberada, no una limitación. En la mayoría de los lenguajes la mutabilidad es el default y la inmutabilidad hay que pedirla (`final` en Java, `const` en C, `readonly` en C#). Rust invierte esa presunción: como el compilador conoce la mutabilidad de cada variable en tiempo de compilación, puede razonar sobre alias (¿quién más puede estar viendo/modificando este dato?) de forma mucho más estricta, lo cual es la base de todo el sistema de seguridad de memoria y concurrencia del lenguaje.

La misma idea aplica a las referencias, con dos formas independientes de "constancia":

```rust
let mut v = vec![1, 2, 3];

let r: &Vec<i32> = &v;        // referencia inmutable: no puedo modificar v a traves de r
let m: &mut Vec<i32> = &mut v; // referencia mutable: si puedo, pero de forma exclusiva
```

## 4.2 Interior mutability: cuando la regla estricta es demasiado estricta

A veces necesitas mutar un valor a través de una referencia compartida (`&T`), por ejemplo para implementar una cache o un contador compartido entre varias partes del código que solo tienen acceso de lectura. Para estos casos, Rust ofrece el patrón de **interior mutability**: tipos que mueven la verificación de las reglas de préstamo de tiempo de compilación a tiempo de ejecución.

### `Cell<T>`: para tipos `Copy`

```rust
use std::cell::Cell;

let c = Cell::new(5);
c.set(10); // muta a traves de una referencia inmutable a `c`
println!("{}", c.get());
```

`Cell` nunca entrega referencias al valor interior: solo permite copiarlo hacia afuera (`get`) o reemplazarlo entero (`set`). Como nunca hay una referencia viva al contenido, no hay forma de violar las reglas de alias — el chequeo es innecesario incluso en runtime.

### `RefCell<T>`: para cualquier tipo, con chequeo en runtime

```rust
use std::cell::RefCell;

let datos = RefCell::new(vec![1, 2, 3]);
datos.borrow_mut().push(4); // prestamo mutable verificado EN RUNTIME
println!("{:?}", datos.borrow());
```

`RefCell` sí permite obtener referencias (`borrow()` → `Ref<T>`, `borrow_mut()` → `RefMut<T>`), pero mueve la verificación de "¿hay ya una referencia mutable viva?" del compilador a un contador interno que se revisa en cada llamada. Si violas la regla —por ejemplo, pides `borrow_mut()` mientras ya existe un `borrow()` vivo— el programa **entra en pánico en tiempo de ejecución** (`already borrowed: BorrowMutError`), en vez de fallar en compilación.

**Esto no es una brecha de seguridad**: la garantía de "nunca dos mutables simultáneas" se sigue cumpliendo, solo que se verifica más tarde. `RefCell` es útil cuando el patrón de acceso es demasiado dinámico para que el borrow checker (que analiza estáticamente) lo pueda probar seguro, pero tú sabes —por la lógica del programa— que en la práctica nunca habrá conflicto.

Un patrón común es combinar `Rc<RefCell<T>>`: `Rc` permite múltiples dueños (ownership compartido, de un solo hilo) y `RefCell` permite mutar a través de esos dueños compartidos.

## 4.3 Concurrencia: "fearless concurrency"

Rust promociona la idea de **fearless concurrency**: el mismo sistema de tipos que previene bugs de memoria en código de un solo hilo, extendido con dos traits marcadores, previene *data races* en código multi-hilo — verificado en tiempo de compilación, no descubierto en producción bajo carga.

### Los dos traits que hacen esto posible

- **`Send`**: un tipo es `Send` si es seguro **transferir su ownership** a otro hilo.
- **`Sync`**: un tipo es `Sync` si es seguro **compartir una referencia** (`&T`) entre varios hilos simultáneamente (formalmente: `T` es `Sync` si y solo si `&T` es `Send`).

Casi todos los tipos primitivos son `Send + Sync`. El ejemplo clásico de lo que **no** lo es: `Rc<T>` no es `Send` ni `Sync`, porque su contador de referencias no está sincronizado (incrementarlo desde dos hilos a la vez sería una data race). El compilador rechaza compilar código que intente enviar un `Rc<T>` a otro hilo — **antes de ejecutar una sola línea**, no como un bug intermitente que aparece bajo carga en producción.

```rust
use std::sync::Arc;
use std::rc::Rc;

// Rc<T> no es Send: esto NO compila si se usa a traves de std::thread::spawn
// let compartido = Rc::new(5);
// std::thread::spawn(move || println!("{compartido}")); // ERROR: `Rc<i32>` cannot be sent between threads

// Arc<T> (Atomic Rc) SI es Send + Sync: usa conteo de referencias atomico
let compartido = Arc::new(5);
let clon = Arc::clone(&compartido);
std::thread::spawn(move || println!("{clon}"));
```

### Threads y `move` closures

```rust
use std::thread;

let v = vec![1, 2, 3];

let handle = thread::spawn(move || {
    println!("vector en el hilo: {v:?}");
});

handle.join().unwrap(); // espera a que el hilo termine
```

El closure `move` toma ownership de `v` y lo mueve al hilo. Esto es exactamente el mismo mecanismo de *move* del capítulo 1, aplicado a través de un límite de hilos: el compilador garantiza que el hilo principal **no puede** seguir usando `v` después de moverlo, así que no hay forma de que ambos hilos accedan al mismo dato sin sincronización.

### `Mutex<T>`: exclusión mutua verificada por tipos

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let contador = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let contador = Arc::clone(&contador);
    let handle = thread::spawn(move || {
        let mut num = contador.lock().unwrap(); // bloquea, devuelve MutexGuard<i32>
        *num += 1;
    }); // el MutexGuard se libera aqui (Drop), desbloqueando el mutex
    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

println!("resultado: {}", *contador.lock().unwrap()); // 10
```

`Mutex<T>` en Rust es distinto de un mutex en C++ o Java: **el dato que protege está dentro del propio mutex** (`Mutex<T>`, no un mutex separado que "por convención" protege una variable). Es imposible acceder a `T` sin pasar por `.lock()`, y es imposible olvidar liberar el lock: `lock()` devuelve un `MutexGuard<T>` cuyo `Drop` libera el lock automáticamente al salir de scope — el mismo patrón RAII del capítulo 1, aplicado a sincronización.

`Arc<Mutex<T>>` es el patrón estándar para estado compartido y mutable entre hilos: `Arc` para ownership compartido con conteo atómico, `Mutex` para exclusión mutua con chequeo en tiempo de ejecución (análogo a `Rc<RefCell<T>>`, pero seguro entre hilos).

### Channels: comunicar en vez de compartir memoria

```rust
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel();

thread::spawn(move || {
    let mensajes = vec!["hola", "desde", "el hilo"];
    for m in mensajes {
        tx.send(m.to_string()).unwrap();
    }
}); // tx se libera aqui, lo que cierra el canal

for recibido in rx {
    println!("recibido: {recibido}");
}
```

Rust sigue el lema de Go: *"no te comuniques compartiendo memoria; comparte memoria comunicándote"*. Un `mpsc::channel` (*multiple producer, single consumer*) permite mover datos de un hilo a otro sin memoria compartida explícita — y como es *move*, el compilador garantiza que el hilo emisor no puede seguir usando el dato después de enviarlo.

## 4.4 Por qué esto es "fearless"

La combinación de:

1. Ownership + move (capítulo 1) aplicado a través de hilos,
2. Los traits `Send`/`Sync` verificados en tiempo de compilación,
3. Tipos como `Mutex<T>` que hacen imposible acceder al dato sin sincronización,

...significa que una clase entera de bugs de concurrencia (data races clásicas: acceso concurrente no sincronizado con al menos una escritura) son **errores de compilación**, no bugs intermitentes que aparecen bajo carga específica en producción y son casi imposibles de reproducir. Esto no elimina otros bugs de concurrencia (deadlocks, race conditions lógicas), pero sí la categoría más común y más difícil de depurar.

## 4.5 Resumen mental

| Herramienta | Qué resuelve | Chequeo |
|---|---|---|
| `mut` | Mutabilidad explícita, presunción de inmutabilidad | Compilación |
| `Cell<T>` | Mutar un `Copy` a través de `&T` | Compilación (sin referencias vivas, no hace falta runtime) |
| `RefCell<T>` | Mutar cualquier tipo a través de `&T`, patrón de acceso dinámico | Runtime (panic si se viola) |
| `Rc<T>` | Ownership compartido, un solo hilo | Compilación (no es `Send`) |
| `Arc<T>` | Ownership compartido entre hilos | Compilación (conteo atómico, es `Send + Sync`) |
| `Mutex<T>` | Mutación exclusiva entre hilos | Runtime (`lock()` bloquea) + compilación (dato inaccesible sin lock) |
| `mpsc::channel` | Transferir datos entre hilos sin memoria compartida | Compilación (move a través del canal) |
