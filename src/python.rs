//! Python bindings (feature `python`; build with maturin). Results are plain dicts.

use crate::api::{self, Request, Variant};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::collections::BTreeMap;

fn to_py(py: Python<'_>, resp: &api::Response) -> PyResult<Py<PyAny>> {
    let json = serde_json::to_string(resp).map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(py.import("json")?.call_method1("loads", (json,))?.unbind())
}

fn exec(py: Python<'_>, req: Request) -> PyResult<Py<PyAny>> {
    // the sim is pure Rust: let other Python threads run meanwhile
    let resp = py.detach(|| api::run(&req)).map_err(PyValueError::new_err)?;
    to_py(py, &resp)
}

/// sim(spec, calib=None, gear_lines=None, iterations=10000, target_error=None, target_error_pct=None, seed=1, set=None, max_iterations=200000)
/// -> {"spec", "iterations", "seconds", "dps", "dps_error_95"}
#[pyfunction]
#[pyo3(signature = (spec, calib=None, gear_lines=None, iterations=10000, target_error=None, target_error_pct=None, seed=1, set=None, max_iterations=200000))]
#[allow(clippy::too_many_arguments)]
fn sim(
    py: Python<'_>,
    spec: String,
    calib: Option<String>,
    gear_lines: Option<Vec<String>>,
    iterations: u64,
    target_error: Option<f64>,
    target_error_pct: Option<f64>,
    seed: u64,
    set: Option<BTreeMap<String, f64>>,
    max_iterations: u64,
) -> PyResult<Py<PyAny>> {
    let req = Request {
        spec, calib, base_gear: gear_lines.unwrap_or_default(), variants: vec![],
        set: set.unwrap_or_default(), iterations, target_error, target_error_pct, max_iterations, seed,
    };
    let out = exec(py, req)?;
    let d = out.bind(py).cast::<pyo3::types::PyDict>()?.clone();
    let base = d.get_item("base")?.unwrap();
    d.del_item("base")?;
    d.del_item("variants")?;
    d.set_item("dps", base.get_item("dps")?)?;
    d.set_item("dps_error_95", base.get_item("dps_error_95")?)?;
    Ok(d.into_any().unbind())
}

/// sim_batch(spec, base_gear, variants, calib=None, iterations=10000, target_error=None, target_error_pct=None, seed=1, set=None, max_iterations=200000)
/// `variants`: list of {"name": str, "gear": [slot lines replacing the base's], "set": {path: value}}.
/// -> {"spec", "iterations", "seconds", "base": {dps, dps_error_95},
///     "variants": [{name, dps, dps_error_95, delta, delta_error_95}]}
#[pyfunction]
#[pyo3(signature = (spec, base_gear, variants, calib=None, iterations=10000, target_error=None, target_error_pct=None, seed=1, set=None, max_iterations=200000))]
#[allow(clippy::too_many_arguments)]
fn sim_batch(
    py: Python<'_>,
    spec: String,
    base_gear: Vec<String>,
    variants: Vec<Bound<'_, PyAny>>,
    calib: Option<String>,
    iterations: u64,
    target_error: Option<f64>,
    target_error_pct: Option<f64>,
    seed: u64,
    set: Option<BTreeMap<String, f64>>,
    max_iterations: u64,
) -> PyResult<Py<PyAny>> {
    let mut vs = Vec::new();
    for v in variants {
        vs.push(Variant {
            name: v.get_item("name")?.extract()?,
            gear: match v.get_item("gear") { Ok(g) => g.extract()?, Err(_) => vec![] },
            set: match v.get_item("set") { Ok(s) => s.extract()?, Err(_) => BTreeMap::new() },
        });
    }
    let req = Request {
        spec, calib, base_gear, variants: vs, set: set.unwrap_or_default(),
        iterations, target_error, target_error_pct, max_iterations, seed,
    };
    exec(py, req)
}

/// Built-in spec names.
#[pyfunction]
fn spec_names() -> Vec<&'static str> { api::spec_names() }

/// True if the spec can compute stats from gear lines.
#[pyfunction]
fn supports_gear(spec: &str) -> bool { api::supports_gear(spec) }

/// gear_stats(spec, gear_lines) -> per-item ilvl/stats and the character's totals (as `azsim --gear-stats`).
#[pyfunction]
fn gear_stats(py: Python<'_>, spec: String, gear_lines: Vec<String>) -> PyResult<Py<PyAny>> {
    let rep = py.detach(|| api::gear_report(&spec, &gear_lines)).map_err(PyValueError::new_err)?;
    let json = serde_json::to_string(&rep).map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(py.import("json")?.call_method1("loads", (json,))?.unbind())
}

#[pymodule]
fn azsim(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sim, m)?)?;
    m.add_function(wrap_pyfunction!(sim_batch, m)?)?;
    m.add_function(wrap_pyfunction!(spec_names, m)?)?;
    m.add_function(wrap_pyfunction!(supports_gear, m)?)?;
    m.add_function(wrap_pyfunction!(gear_stats, m)?)?;
    Ok(())
}
