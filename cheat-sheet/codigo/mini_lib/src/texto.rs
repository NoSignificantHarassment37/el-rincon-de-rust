//! Modulo de utilidades de texto.

pub fn invertir(s: &str) -> String {
    s.chars().rev().collect()
}

pub fn es_palindromo(s: &str) -> bool {
    let normalizado: String = s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();
    normalizado == invertir(&normalizado)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invertir_una_palabra() {
        assert_eq!(invertir("rust"), "tsur");
    }

    #[test]
    fn palindromo_simple() {
        assert!(es_palindromo("reconocer"));
        assert!(!es_palindromo("rust"));
    }

    #[test]
    fn palindromo_ignora_espacios_y_mayusculas() {
        assert!(es_palindromo("Anita lava la tina"));
    }
}
