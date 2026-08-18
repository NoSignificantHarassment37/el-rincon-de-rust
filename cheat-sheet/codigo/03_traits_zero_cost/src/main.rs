//! Ejemplos ejecutables de traits, static vs dynamic dispatch y zero-cost abstractions.
//! Ver la explicacion conceptual en `cheat-sheet/conceptos/03-traits-zero-cost-abstractions.md`.

fn main() {
    ejemplo_trait_basico();
    ejemplo_static_dispatch();
    ejemplo_dynamic_dispatch();
    ejemplo_generic_el_mayor();
    ejemplo_derive();
    ejemplo_iteradores_zero_cost();
}

/// Un trait define comportamiento compartido. `resumen_largo` tiene una
/// implementacion por defecto que los tipos pueden heredar o sobreescribir.
trait Resumible {
    fn resumen(&self) -> String;

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
        let corte = 20.min(self.contenido.len());
        format!("{}: {}...", self.titular, &self.contenido[..corte])
    }
}

struct Tweet {
    usuario: String,
    texto: String,
}

impl Resumible for Tweet {
    fn resumen(&self) -> String {
        format!("@{}: {}", self.usuario, self.texto)
    }
    // Sobreescribe la implementacion por defecto de resumen_largo:
    fn resumen_largo(&self) -> String {
        format!("[tweet] {}", self.resumen())
    }
}

fn ejemplo_trait_basico() {
    println!("--- trait basico con default method ---");
    let articulo = Articulo {
        titular: String::from("Rust 2.0"),
        contenido: String::from("Hoy se anuncio una nueva version del lenguaje..."),
    };
    println!("{}", articulo.resumen());
    println!("{}", articulo.resumen_largo()); // usa el default

    let tweet = Tweet {
        usuario: String::from("rustlang"),
        texto: String::from("zero-cost abstractions, siempre"),
    };
    println!("{}", tweet.resumen_largo()); // usa la version sobreescrita
}

/// Static dispatch: el compilador genera una copia especializada de esta
/// funcion por cada tipo concreto (monomorphization). No hay indireccion
/// en tiempo de ejecucion: es tan rapido como si hubieras escrito una
/// funcion distinta a mano para cada tipo.
fn notificar_estatico(item: &impl Resumible) {
    println!("[static] {}", item.resumen());
}

fn ejemplo_static_dispatch() {
    println!("--- static dispatch (impl Trait) ---");
    let articulo = Articulo {
        titular: String::from("Ownership explicado"),
        contenido: String::from("El ownership es el corazon de Rust..."),
    };
    let tweet = Tweet {
        usuario: String::from("ferris"),
        texto: String::from("no gc, no data races"),
    };
    notificar_estatico(&articulo); // monomorphization: version para Articulo
    notificar_estatico(&tweet);    // monomorphization: version para Tweet
}

/// Dynamic dispatch: `Box<dyn Resumible>` permite guardar tipos distintos
/// en una misma coleccion homogenea, a costa de una indireccion (vtable)
/// por llamada. Esto es lo que hace posible la heterogeneidad, pero ya
/// no es "zero-cost" en sentido estricto.
fn ejemplo_dynamic_dispatch() {
    println!("--- dynamic dispatch (dyn Trait) ---");
    let items: Vec<Box<dyn Resumible>> = vec![
        Box::new(Articulo {
            titular: String::from("Traits en profundidad"),
            contenido: String::from("Los traits permiten polimorfismo..."),
        }),
        Box::new(Tweet {
            usuario: String::from("crab"),
            texto: String::from("dyn Trait = vtable"),
        }),
    ];
    for item in &items {
        println!("[dyn] {}", item.resumen());
    }
}

/// Generico con trait bounds: `T: PartialOrd + Copy` es el contrato minimo
/// necesario para poder comparar (`>`) y copiar valores de tipo T.
fn el_mayor<T: PartialOrd + Copy>(lista: &[T]) -> T {
    let mut mayor = lista[0];
    for &item in lista {
        if item > mayor {
            mayor = item;
        }
    }
    mayor
}

fn ejemplo_generic_el_mayor() {
    println!("--- generico con trait bounds ---");
    let enteros = [3, 7, 2, 9, 4];
    let flotantes = [3.1, 7.4, 2.0, 9.9];
    println!("mayor entero: {}", el_mayor(&enteros));
    println!("mayor flotante: {}", el_mayor(&flotantes));
    assert_eq!(el_mayor(&enteros), 9);
}

/// La mayoria de los traits estandar se derivan automaticamente.
#[derive(Debug, Clone, PartialEq, Default)]
struct Punto {
    x: i32,
    y: i32,
}

fn ejemplo_derive() {
    println!("--- #[derive(...)] ---");
    let p1 = Punto { x: 1, y: 2 };
    let p2 = p1.clone();
    let p3 = Punto::default();
    println!("{p1:?} == {p2:?}: {}", p1 == p2);
    println!("default: {p3:?}");
    assert_eq!(p1, p2);
    assert_eq!(p3, Punto { x: 0, y: 0 });
}

/// Los iteradores son la zero-cost abstraction por excelencia: esta cadena
/// declarativa se compila a un bucle tan eficiente como su version imperativa.
fn ejemplo_iteradores_zero_cost() {
    println!("--- iteradores (zero-cost) ---");
    let suma_declarativa: i32 = (1..=100).filter(|n| n % 2 == 0).map(|n| n * n).sum();

    let mut suma_imperativa = 0;
    for n in 1..=100 {
        if n % 2 == 0 {
            suma_imperativa += n * n;
        }
    }

    println!("declarativo = {suma_declarativa}, imperativo = {suma_imperativa}");
    assert_eq!(suma_declarativa, suma_imperativa);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resumen_de_articulo() {
        let a = Articulo {
            titular: String::from("T"),
            contenido: String::from("contenido de prueba largo"),
        };
        assert!(a.resumen().starts_with("T:"));
    }

    #[test]
    fn resumen_largo_por_defecto_vs_sobreescrito() {
        let a = Articulo {
            titular: String::from("T"),
            contenido: String::from("c"),
        };
        let t = Tweet {
            usuario: String::from("u"),
            texto: String::from("hola"),
        };
        assert!(a.resumen_largo().starts_with("(leer mas...)"));
        assert!(t.resumen_largo().starts_with("[tweet]"));
    }

    #[test]
    fn el_mayor_enteros() {
        assert_eq!(el_mayor(&[1, 5, 3]), 5);
    }

    #[test]
    fn el_mayor_flotantes() {
        assert_eq!(el_mayor(&[1.5, 0.2, 3.3]), 3.3);
    }

    #[test]
    fn derive_default_y_eq() {
        assert_eq!(Punto::default(), Punto { x: 0, y: 0 });
        assert_eq!(Punto { x: 1, y: 1 }.clone(), Punto { x: 1, y: 1 });
    }

    #[test]
    fn iteradores_igualan_bucle_imperativo() {
        let a: i32 = (1..=10).filter(|n| n % 2 == 0).map(|n| n * n).sum();
        let mut b = 0;
        for n in 1..=10 {
            if n % 2 == 0 {
                b += n * n;
            }
        }
        assert_eq!(a, b);
    }
}
