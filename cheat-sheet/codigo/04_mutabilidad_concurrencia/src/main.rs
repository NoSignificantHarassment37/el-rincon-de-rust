//! Ejemplos ejecutables de mutabilidad explicita, interior mutability y concurrencia.
//! Ver la explicacion conceptual en `cheat-sheet/conceptos/04-mutabilidad-concurrencia.md`.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

fn main() {
    ejemplo_mut_explicito();
    ejemplo_cell();
    ejemplo_refcell();
    ejemplo_rc_refcell_compartido();
    ejemplo_threads_con_move();
    ejemplo_mutex_arc();
    ejemplo_channels();
}

/// Todo es inmutable por defecto; `mut` es una declaracion explicita.
fn ejemplo_mut_explicito() {
    println!("--- mut explicito ---");
    let x = 5;
    // x = 6; // error[E0384]: cannot assign twice to immutable variable

    let mut y = 5;
    println!("y antes de mutar = {y}");
    y = 6;
    println!("x = {x}, y (mutado) = {y}");
}

/// Cell<T>: muta un valor Copy a traves de una referencia inmutable.
/// Nunca entrega referencias al contenido, asi que no hace falta chequeo
/// en runtime: solo permite get() (copia) y set() (reemplazo completo).
fn ejemplo_cell() {
    println!("--- Cell<T> ---");
    let c = Cell::new(5);
    c.set(10);
    println!("c = {}", c.get());
    assert_eq!(c.get(), 10);
}

/// RefCell<T>: permite pedir prestamos mutables/inmutables a traves de una
/// referencia compartida, verificados EN RUNTIME (panic si se violan).
fn ejemplo_refcell() {
    println!("--- RefCell<T> ---");
    let datos = RefCell::new(vec![1, 2, 3]);
    datos.borrow_mut().push(4);
    println!("datos = {:?}", datos.borrow());
    assert_eq!(*datos.borrow(), vec![1, 2, 3, 4]);

    // Violar la regla en runtime causa panic, no error de compilacion:
    // let _r1 = datos.borrow_mut();
    // let _r2 = datos.borrow_mut(); // panic: already borrowed: BorrowMutError
}

/// Rc<RefCell<T>>: ownership compartido (un solo hilo) + mutabilidad
/// interior. Patron comun para estructuras con multiples "duenos" logicos.
fn ejemplo_rc_refcell_compartido() {
    println!("--- Rc<RefCell<T>> ---");
    let compartido = Rc::new(RefCell::new(vec![1, 2, 3]));

    let clon_a = Rc::clone(&compartido);
    let clon_b = Rc::clone(&compartido);

    clon_a.borrow_mut().push(4);
    clon_b.borrow_mut().push(5);

    println!("compartido = {:?}", compartido.borrow());
    println!("Rc::strong_count = {}", Rc::strong_count(&compartido));
    assert_eq!(*compartido.borrow(), vec![1, 2, 3, 4, 5]);
    assert_eq!(Rc::strong_count(&compartido), 3);
}

/// `move` transfiere el ownership de `v` al hilo. El hilo principal ya no
/// puede usar `v` despues de este punto: el compilador lo garantiza.
fn ejemplo_threads_con_move() {
    println!("--- threads con move ---");
    let v = vec![1, 2, 3];

    let handle = thread::spawn(move || {
        println!("vector en el hilo: {v:?}");
        v.iter().sum::<i32>()
    });

    let suma = handle.join().unwrap();
    println!("suma calculada en el hilo: {suma}");
    assert_eq!(suma, 6);
}

/// Arc<Mutex<T>>: ownership compartido entre hilos (conteo atomico) +
/// exclusion mutua verificada. El dato esta DENTRO del mutex: es
/// imposible acceder a el sin pasar por lock().
fn ejemplo_mutex_arc() {
    println!("--- Arc<Mutex<T>> ---");
    let contador = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let contador = Arc::clone(&contador);
        let handle = thread::spawn(move || {
            let mut num = contador.lock().unwrap();
            *num += 1;
        }); // el MutexGuard se libera aqui (Drop), desbloqueando el mutex
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let resultado = *contador.lock().unwrap();
    println!("resultado final: {resultado}");
    assert_eq!(resultado, 10);
}

/// mpsc::channel: comunicar moviendo datos entre hilos, en vez de
/// compartir memoria directamente.
fn ejemplo_channels() {
    println!("--- mpsc::channel ---");
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mensajes = vec!["hola", "desde", "el hilo"];
        for m in mensajes {
            tx.send(m.to_string()).unwrap();
        }
    }); // tx se libera aqui, lo que cierra el canal

    let recibidos: Vec<String> = rx.iter().collect();
    for m in &recibidos {
        println!("recibido: {m}");
    }
    assert_eq!(recibidos, vec!["hola", "desde", "el hilo"]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_permite_mutar_via_referencia_inmutable() {
        let c = Cell::new(1);
        let r = &c;
        r.set(2);
        assert_eq!(c.get(), 2);
    }

    #[test]
    fn refcell_permite_prestamo_mutable_en_runtime() {
        let datos = RefCell::new(vec![1]);
        datos.borrow_mut().push(2);
        assert_eq!(*datos.borrow(), vec![1, 2]);
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn refcell_viola_la_regla_en_runtime_con_panic() {
        let datos = RefCell::new(5);
        let _r1 = datos.borrow_mut();
        let _r2 = datos.borrow_mut(); // panic esperado
    }

    #[test]
    fn rc_strong_count_crece_con_cada_clone() {
        let a = Rc::new(5);
        let _b = Rc::clone(&a);
        let _c = Rc::clone(&a);
        assert_eq!(Rc::strong_count(&a), 3);
    }

    #[test]
    fn thread_devuelve_resultado_via_join() {
        let handle = thread::spawn(|| 2 + 2);
        assert_eq!(handle.join().unwrap(), 4);
    }

    #[test]
    fn mutex_sincroniza_incrementos_concurrentes() {
        let contador = Arc::new(Mutex::new(0));
        let mut handles = vec![];
        for _ in 0..20 {
            let contador = Arc::clone(&contador);
            handles.push(thread::spawn(move || {
                *contador.lock().unwrap() += 1;
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(*contador.lock().unwrap(), 20);
    }

    #[test]
    fn channel_transfiere_datos_entre_hilos() {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || tx.send(42).unwrap());
        assert_eq!(rx.recv().unwrap(), 42);
    }
}
