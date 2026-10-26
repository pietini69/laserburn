pub mod camera;
pub mod device;
pub mod dither;
pub mod document;
pub mod formats;
pub mod geometry;
pub mod hatch;
pub mod material;
pub mod optimizer;
pub mod simulator;
pub mod text;

#[cfg(feature = "gui")]
pub mod gui;

pub use document::{CutMode, CutSetting, DitherMode, Document, Layer, Shape};
pub use material::{MaterialEntry, MaterialLibrary, MaterialOperation, ThicknessEntry};
pub use formats::{GCodeConfig, GCodeGenerator, Lbrn2File, RuidaGenerator, SvgFormat};
pub use geometry::{BoundingBox, Point2D, Polyline, Vector2D};
#[cfg(feature = "gui")]
pub use gui::LaserBurnApp;
pub use hatch::generate_hatch;
pub use optimizer::{CutOptimizer, OptimizedJob, OptimizedPath};
pub use simulator::{SimulationStats, Simulator};
