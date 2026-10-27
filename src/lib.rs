pub mod camera;
pub mod device;
pub mod dither;
pub mod document;
pub mod formats;
pub mod geometry;
pub mod hatch;
pub mod i18n;
pub mod material;
pub mod optimizer;
pub mod simulator;
pub mod text;
pub mod trace;
pub mod nesting;
pub mod server;

#[cfg(feature = "gui")]
pub mod gui;

pub use document::{CutMode, CutSetting, DitherMode, Document, Layer, Shape};
pub use i18n::Language;
pub use material::{MaterialEntry, MaterialLibrary, MaterialOperation, ThicknessEntry};
pub use formats::{DxfFormat, GCodeConfig, GCodeGenerator, Lbrn2File, PdfFormat, RuidaGenerator, SvgFormat};
pub use geometry::{BoundingBox, Point2D, Polyline, Vector2D};
pub use trace::{TraceOptions, trace_image_to_polylines};
pub use server::BridgeServer;
#[cfg(feature = "gui")]
pub use gui::LaserBurnApp;
#[cfg(feature = "gui")]
pub use gui::LaserBurnApp as PhotonCamApp;
pub use hatch::generate_hatch;
pub use optimizer::{CutOptimizer, OptimizedJob, OptimizedPath};
pub use simulator::{SimulationStats, Simulator};
