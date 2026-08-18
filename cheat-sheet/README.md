# Rust Cheat Sheet — Conceptos y Código

Material de apoyo para acompañar la lectura del libro oficial de Rust ("The Book"), organizado en 5 grupos temáticos. Cada grupo tiene:

- Una explicación conceptual extensa en [`conceptos/`](conceptos/), en Markdown.
- Un proyecto de Rust ejecutable y con tests en [`codigo/`](codigo/), dentro de un [cargo workspace](conceptos/05-modulos-crates-cargo.md#58-workspaces-varios-packages-un-solo-cargolock).

| # | Conceptos | Markdown | Código |
|---|---|---|---|
| 1 | Ownership, Borrowing y Referencias | [conceptos/01](conceptos/01-ownership-borrowing-referencias.md) | [codigo/01_ownership_borrowing](codigo/01_ownership_borrowing/src/main.rs) |
| 2 | Lifetimes, Tipos algebraicos y Pattern Matching | [conceptos/02](conceptos/02-lifetimes-pattern-matching.md) | [codigo/02_lifetimes_pattern_matching](codigo/02_lifetimes_pattern_matching/src/main.rs) |
| 3 | Traits y Zero-cost Abstractions | [conceptos/03](conceptos/03-traits-zero-cost-abstractions.md) | [codigo/03_traits_zero_cost](codigo/03_traits_zero_cost/src/main.rs) |
| 4 | Mutabilidad Explícita y Concurrencia | [conceptos/04](conceptos/04-mutabilidad-concurrencia.md) | [codigo/04_mutabilidad_concurrencia](codigo/04_mutabilidad_concurrencia/src/main.rs) |
| 5 | Módulos, Crates y Cargo | [conceptos/05](conceptos/05-modulos-crates-cargo.md) | [codigo/05_modulos_crates_cargo](codigo/05_modulos_crates_cargo/src/main.rs) + [codigo/mini_lib](codigo/mini_lib/src/lib.rs) |

## Cómo correr el código

Todo `codigo/*` es un [workspace](conceptos/05-modulos-crates-cargo.md) de Cargo, así que los comandos se corren desde `cheat-sheet/codigo/`:

```bash
cd cheat-sheet/codigo

# Correr un ejemplo especifico:
cargo run -p ownership_borrowing
cargo run -p lifetimes_pattern_matching
cargo run -p traits_zero_cost
cargo run -p mutabilidad_concurrencia
cargo run -p modulos_crates_cargo

# Correr TODOS los tests de TODOS los ejemplos:
cargo test --workspace

# Verificar que todo compile sin generar binarios (rapido):
cargo check --workspace
```

Cada `src/main.rs` está comentado y dividido en funciones `ejemplo_*()`, una por sub-tema, con asserts inline y una sección `#[cfg(test)] mod tests` con tests unitarios adicionales. La idea es leer el `.md` correspondiente primero y luego correr/editar el código para experimentar.
