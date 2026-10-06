//! The games, as Python sees them: a picture in, a few lines of action out.

use minigame::catalog::CATALOG;
use minigame::frame::{Frame, HEIGHT, WIDTH};
use minigame::game::{LIMIT, Scoring, TICK_HZ};
use minigame::input::Action;
use minigame::replay::replay;
use minigame::session::Session;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

#[pyclass(unsendable, module = "minigames._core")]
struct Episode {
    session: Session,
    canvas: Frame,
    drawn: bool,
}

#[pymethods]
impl Episode {
    #[new]
    #[pyo3(signature = (game, seed = 0))]
    fn new(game: &str, seed: u64) -> PyResult<Self> {
        let session = Session::new(game, seed).ok_or_else(|| {
            PyValueError::new_err(format!(
                "no game called {game}. Known games: {}",
                minigame::catalog::names().collect::<Vec<_>>().join(", ")
            ))
        })?;
        Ok(Episode { session, canvas: Frame::new(), drawn: false })
    }

    #[getter]
    fn game(&self) -> &'static str {
        self.session.spec().name
    }

    #[getter]
    fn prompt(&self) -> String {
        self.session.spec().prompt()
    }

    #[getter]
    fn seed(&self) -> u64 {
        self.session.seed()
    }

    #[getter]
    fn outcome(&self) -> &'static str {
        self.session.outcome().name()
    }

    #[getter]
    fn tick(&self) -> u32 {
        self.session.tick()
    }

    #[getter]
    fn seconds(&self) -> f32 {
        self.session.seconds()
    }

    /// The screen as a PNG.
    fn frame<'py>(&mut self, py: Python<'py>) -> Bound<'py, PyBytes> {
        self.draw();
        PyBytes::new(py, &self.canvas.to_png())
    }

    /// The screen as `WIDTH * HEIGHT * 3` bytes of red, green, and blue.
    fn pixels<'py>(&mut self, py: Python<'py>) -> Bound<'py, PyBytes> {
        self.draw();
        PyBytes::new(py, &rgb(&self.canvas))
    }

    /// Perform the actions, let `ticks` of game time pass, and say where it
    /// stands.
    #[pyo3(signature = (*actions, ticks = 1))]
    fn act(&mut self, actions: Vec<String>, ticks: u32) -> PyResult<&'static str> {
        let parsed = actions
            .iter()
            .map(|line| Action::parse(line))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let outcome = self.session.advance(&parsed, ticks);
        self.drawn = false;
        Ok(outcome.name())
    }

    /// Play the episode again from its seed, handing `each` the screen at its
    /// start and after every tick as red, green, and blue bytes. A replay that
    /// does not end where the episode did is an error.
    fn replay<'py>(&self, py: Python<'py>, each: Bound<'py, PyAny>) -> PyResult<()> {
        let fresh = Session::new(self.game(), self.seed()).expect("it was played");
        let again = replay(fresh, self.session.record(), self.session.tick(), |frame: &Frame| {
            each.call1((PyBytes::new(py, &rgb(frame)),)).map(drop)
        })?;
        if (again.outcome(), again.tick()) != (self.session.outcome(), self.session.tick()) {
            return Err(PyRuntimeError::new_err(
                "played again, the episode did not come out the same",
            ));
        }
        Ok(())
    }

    fn report<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let report = self.session.report();
        let out = PyDict::new(py);
        out.set_item("game", report.game)?;
        out.set_item("seed", self.session.seed())?;
        out.set_item("outcome", report.outcome.name())?;
        out.set_item("ticks", self.session.tick())?;
        Ok(out)
    }

    fn __repr__(&self) -> String {
        format!(
            "Episode({}, seed={}, {} at {:.2}s)",
            self.game(),
            self.session.seed(),
            self.outcome(),
            self.seconds()
        )
    }
}

impl Episode {
    /// Draw the screen into the canvas, unless nothing has happened since it last was.
    fn draw(&mut self) {
        if !self.drawn {
            self.session.draw(&mut self.canvas);
            self.drawn = true;
        }
    }
}

/// A frame as `WIDTH * HEIGHT * 3` bytes of red, green, and blue.
fn rgb(frame: &Frame) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((WIDTH * HEIGHT * 3) as usize);
    for packed in frame.pixels() {
        bytes.extend_from_slice(&[(packed >> 16) as u8, (packed >> 8) as u8, *packed as u8]);
    }
    bytes
}

fn scoring_name(scoring: Scoring) -> &'static str {
    match scoring {
        Scoring::Boolean => "finishing",
        Scoring::Time => "time",
    }
}

/// Play `game` on `seed` again through `record`, each line of the action language beside the
/// tick it landed at the start of, until `end` ticks have passed or the round is over, handing
/// `each`, where given, the screen at the start and after every tick as red, green, and blue
/// bytes. Says how the round ended and on which tick, for the caller to compare with how it
/// ended when it was played.
#[pyfunction(name = "replay")]
#[pyo3(signature = (game, seed, record, end, each = None))]
fn replay_record(
    py: Python<'_>,
    game: &str,
    seed: u64,
    record: Vec<(u32, String)>,
    end: u32,
    each: Option<Bound<'_, PyAny>>,
) -> PyResult<(&'static str, u32)> {
    let session = Session::new(game, seed)
        .ok_or_else(|| PyValueError::new_err(format!("no game called {game}")))?;
    let landed = record
        .iter()
        .map(|(tick, line)| Action::parse(line).map(|action| (*tick, action)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    let again = replay(session, &landed, end, |frame: &Frame| match &each {
        Some(each) => each.call1((PyBytes::new(py, &rgb(frame)),)).map(drop),
        None => Ok(()),
    })?;
    Ok((again.outcome().name(), again.tick()))
}

/// Everything in the catalog, one dictionary each.
#[pyfunction]
fn games(py: Python<'_>) -> PyResult<Vec<Bound<'_, PyDict>>> {
    CATALOG
        .iter()
        .map(|entry| {
            let spec = &entry.spec;
            let out = PyDict::new(py);
            out.set_item("name", spec.name)?;
            out.set_item("title", spec.title)?;
            out.set_item("brief", spec.brief)?;
            out.set_item("controls", spec.controls.to_vec())?;
            out.set_item("pointer", spec.pointer)?;
            out.set_item("scoring", scoring_name(spec.scoring))?;
            out.set_item("tags", spec.tags.iter().map(|tag| tag.name()).collect::<Vec<_>>())?;
            out.set_item("limit", LIMIT)?;
            out.set_item("prompt", spec.prompt())?;
            Ok(out)
        })
        .collect()
}

/// The width and height every game is drawn at.
#[pyfunction]
fn size() -> (u32, u32) {
    (WIDTH, HEIGHT)
}

/// Ticks of game time in one second of it. Every game is written to this.
#[pyfunction]
fn tick_hz() -> u32 {
    TICK_HZ
}

#[pymodule]
fn _core(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Episode>()?;
    module.add_function(wrap_pyfunction!(games, module)?)?;
    module.add_function(wrap_pyfunction!(replay_record, module)?)?;
    module.add_function(wrap_pyfunction!(size, module)?)?;
    module.add_function(wrap_pyfunction!(tick_hz, module)?)?;
    module
        .add("__all__", vec!["Episode", "games", "replay", "size", "tick_hz"])
        .map_err(PyRuntimeError::new_err)
}
