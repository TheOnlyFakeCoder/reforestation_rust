//! Parámetros del modelo, compartidos por Rust y Python vía `config/default.json`.

use std::error::Error;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Distancia entre vecinos inmediatos del tresbolillo que da `densidad` plantas por hectárea.
pub fn d_por_densidad(densidad: f64) -> f64 {
    (10_000.0 / (0.866 * densidad)).sqrt()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Distancia entre vecinos inmediatos (m).
    pub d: f64,
    /// Holgura ε de la banda de composición.
    pub epsilon: f64,
    /// Peso κ de la similitud entre especies distintas.
    pub kappa: f64,
    /// Multiplicador de todos los radios R_j (sensibilidad 0.75 y 1.25).
    pub factor_radios: f64,
    /// Tolerancia numérica al comparar distancias (m).
    pub tolerancia: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            d: d_por_densidad(658.0),
            epsilon: 0.10,
            kappa: 0.6,
            factor_radios: 1.0,
            tolerancia: 1e-6,
        }
    }
}

impl Config {
    /// Lee y valida un archivo de configuración.
    pub fn cargar(path: &Path) -> Result<Config, Box<dyn Error>> {
        let config = Config::leer(path)?;
        config
            .validar()
            .map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(config)
    }

    /// Lee un archivo sin validarlo; el CLI valida después de aplicar los `--set`.
    pub fn leer(path: &Path) -> Result<Config, Box<dyn Error>> {
        let texto =
            std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(serde_json::from_str(&texto).map_err(|e| format!("{}: {e}", path.display()))?)
    }

    /// Revisa que los parámetros tengan sentido; el mensaje nombra el campo que falla.
    pub fn validar(&self) -> Result<(), String> {
        // `is_nan()` explícito: NaN no es > 0 pero tampoco es <= 0.
        let no_positivo = |x: f64| x.is_nan() || x <= 0.0;
        if no_positivo(self.d) {
            return Err(format!("d debe ser > 0 (vale {})", self.d));
        }
        if no_positivo(self.tolerancia) {
            return Err(format!("tolerancia debe ser > 0 (vale {})", self.tolerancia));
        }
        // Los anillos más cercanos (√3·d y 2d) distan 0.27·d; una tolerancia mayor los juntaría.
        if self.tolerancia >= 0.01 * self.d {
            return Err(format!(
                "tolerancia debe ser < 0.01·d = {} (vale {})",
                0.01 * self.d,
                self.tolerancia
            ));
        }
        if !(0.0..1.0).contains(&self.epsilon) {
            return Err(format!("epsilon debe estar en [0, 1) (vale {})", self.epsilon));
        }
        if self.kappa.is_nan() || self.kappa < 0.0 {
            return Err(format!("kappa debe ser ≥ 0 (vale {})", self.kappa));
        }
        if no_positivo(self.factor_radios) {
            return Err(format!("factor_radios debe ser > 0 (vale {})", self.factor_radios));
        }
        Ok(())
    }

    /// Aplica todas las asignaciones `clave=valor` y valida una sola vez al final, de modo que
    /// los `--set` puedan corregir un archivo inválido. Si algo falla, nada cambia.
    pub fn aplicar_todas(&mut self, asignaciones: &[&str]) -> Result<(), String> {
        let mut json = serde_json::to_value(&*self).map_err(|e| e.to_string())?;
        let objeto = json.as_object_mut().expect("Config se serializa como objeto");
        for asignacion in asignaciones {
            let (clave, valor) = asignacion
                .split_once('=')
                .ok_or_else(|| format!("se esperaba clave=valor: {asignacion}"))?;
            let clave = clave.trim();
            if !objeto.contains_key(clave) {
                return Err(format!("clave desconocida: {clave}"));
            }
            let valor: serde_json::Value =
                serde_json::from_str(valor.trim()).map_err(|e| format!("{asignacion}: {e}"))?;
            objeto.insert(clave.to_string(), valor);
        }
        let nueva: Config = serde_json::from_value(json).map_err(|e| e.to_string())?;
        nueva.validar()?;
        *self = nueva;
        Ok(())
    }

    /// Sobrescribe un campo con `"clave=valor"` (el valor se interpreta como JSON) y valida.
    pub fn aplicar(&mut self, asignacion: &str) -> Result<(), String> {
        self.aplicar_todas(&[asignacion])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d_por_defecto_es_4_1892_m() {
        assert!((Config::default().d - 4.189171).abs() < 1e-6);
    }

    #[test]
    fn json_parcial_completa_con_defaults() {
        let config: Config = serde_json::from_str(r#"{"kappa": 0.4}"#).unwrap();
        assert_eq!(config.kappa, 0.4);
        assert_eq!(config.epsilon, 0.10);
        assert_eq!(config.factor_radios, 1.0);
    }

    #[test]
    fn clave_desconocida_es_error() {
        assert!(serde_json::from_str::<Config>(r#"{"kapa": 0.4}"#).is_err());
    }

    #[test]
    fn aplicar_sobrescribe_un_campo() {
        let mut config = Config::default();
        config.aplicar("epsilon=0").unwrap();
        assert_eq!(config.epsilon, 0.0);
        config.aplicar("factor_radios=1.25").unwrap();
        assert_eq!(config.factor_radios, 1.25);
    }

    #[test]
    fn aplicar_rechaza_clave_desconocida_o_sin_igual() {
        let mut config = Config::default();
        assert!(config.aplicar("radio=2").is_err());
        assert!(config.aplicar("epsilon").is_err());
        assert_eq!(config, Config::default());
    }

    #[test]
    fn validar_rechaza_d_no_positivo() {
        assert!(Config::default().validar().is_ok());
        let config = Config {
            d: 0.0,
            ..Config::default()
        };
        let error = config.validar().unwrap_err();
        assert!(error.contains("d"), "{error}");
    }

    #[test]
    fn validar_rechaza_tolerancia_no_positiva() {
        let config = Config {
            tolerancia: -1.0,
            ..Config::default()
        };
        let error = config.validar().unwrap_err();
        assert!(error.contains("tolerancia"), "{error}");
    }

    #[test]
    fn validar_rechaza_epsilon_fuera_de_0_a_1() {
        for epsilon in [-0.1, 1.0] {
            let config = Config {
                epsilon,
                ..Config::default()
            };
            let error = config.validar().unwrap_err();
            assert!(error.contains("epsilon"), "{error}");
        }
        let cero = Config {
            epsilon: 0.0,
            ..Config::default()
        };
        assert!(cero.validar().is_ok());
    }

    #[test]
    fn validar_rechaza_kappa_negativo() {
        let config = Config {
            kappa: -0.5,
            ..Config::default()
        };
        let error = config.validar().unwrap_err();
        assert!(error.contains("kappa"), "{error}");
    }

    #[test]
    fn validar_rechaza_factor_radios_no_positivo() {
        let config = Config {
            factor_radios: 0.0,
            ..Config::default()
        };
        let error = config.validar().unwrap_err();
        assert!(error.contains("factor_radios"), "{error}");
    }

    #[test]
    fn aplicar_valida_y_no_cambia_nada_si_el_valor_es_invalido() {
        let mut config = Config::default();
        let error = config.aplicar("d=0").unwrap_err();
        assert!(error.contains("d"), "{error}");
        assert_eq!(config, Config::default());
    }

    #[test]
    fn cargar_valida_el_archivo() {
        let path = std::env::temp_dir().join(format!("reto_cfg_{}.json", std::process::id()));
        std::fs::write(&path, r#"{"epsilon": 1.5}"#).unwrap();
        let error = Config::cargar(&path).unwrap_err().to_string();
        assert!(error.contains("epsilon"), "{error}");
    }

    #[test]
    fn cargar_archivo_inexistente_es_error() {
        assert!(Config::cargar(std::path::Path::new("no/existe.json")).is_err());
    }

    #[test]
    fn leer_no_valida_para_que_los_set_puedan_corregir() {
        let path = std::env::temp_dir().join(format!("reto_leer_{}.json", std::process::id()));
        std::fs::write(&path, r#"{"epsilon": 1.5}"#).unwrap();
        let config = Config::leer(&path).unwrap();
        assert_eq!(config.epsilon, 1.5);
    }

    #[test]
    fn validar_rechaza_tolerancia_que_juntaria_anillos() {
        // Los anillos más cercanos (√3·d y 2d) están a 0.27·d; tolerancia = 0.1·d ya es absurda.
        let base = Config::default();
        let config = Config {
            tolerancia: 0.1 * base.d,
            ..base
        };
        let error = config.validar().unwrap_err();
        assert!(error.contains("tolerancia"), "{error}");
    }
}
