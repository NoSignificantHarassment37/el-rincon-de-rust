//! Modulo de geometria basica.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangulo {
    pub ancho: f64,
    pub alto: f64,
}

impl Rectangulo {
    pub fn new(ancho: f64, alto: f64) -> Self {
        Rectangulo { ancho, alto }
    }

    pub fn area(&self) -> f64 {
        self.ancho * self.alto
    }

    pub fn perimetro(&self) -> f64 {
        2.0 * (self.ancho + self.alto)
    }

    pub fn es_cuadrado(&self) -> bool {
        (self.ancho - self.alto).abs() < f64::EPSILON
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_de_rectangulo() {
        assert_eq!(Rectangulo::new(3.0, 4.0).area(), 12.0);
    }

    #[test]
    fn cuadrado_se_detecta_correctamente() {
        assert!(Rectangulo::new(5.0, 5.0).es_cuadrado());
        assert!(!Rectangulo::new(5.0, 4.0).es_cuadrado());
    }
}
