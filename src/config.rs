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
    pub fn cargar(path: &Path) -> Result<Config, Box<dyn Error>> {
        let texto =
            std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(serde_json::from_str(&texto).map_err(|e| format!("{}: {e}", path.display()))?)
    }

    /// Sobrescribe un campo con `"clave=valor"`; el valor se interpreta como JSON.
    pub fn aplicar(&mut self, asignacion: &str) -> Result<(), String> {
        let (clave, valor) = asignacion
            .split_once('=')
            .ok_or_else(|| format!("se esperaba clave=valor: {asignacion}"))?;
        let clave = clave.trim();
        let valor: serde_json::Value =
            serde_json::from_str(valor.trim()).map_err(|e| format!("{asignacion}: {e}"))?;
        let mut json = serde_json::to_value(&*self).map_err(|e| e.to_string())?;
        let objeto = json.as_object_mut().expect("Config se serializa como objeto");
        if !objeto.contains_key(clave) {
            return Err(format!("clave desconocida: {clave}"));
        }
        objeto.insert(clave.to_string(), valor);
        *self = serde_json::from_value(json).map_err(|e| format!("{asignacion}: {e}"))?;
        Ok(())
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
    fn cargar_archivo_inexistente_es_error() {
        assert!(Config::cargar(std::path::Path::new("no/existe.json")).is_err());
    }
}
