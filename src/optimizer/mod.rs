use crate::document::Document;
use crate::geometry::{Point2D, Polyline};

#[derive(Debug, Clone)]
pub struct OptimizedJob {
    pub paths: Vec<OptimizedPath>,
}

#[derive(Debug, Clone)]
pub struct OptimizedPath {
    pub layer_index: usize,
    pub polyline: Polyline,
    pub is_inner_cut: bool,
}

pub struct CutOptimizer {
    pub inner_cuts_first: bool,
    pub minimize_travel: bool,
    pub allow_reversal: bool,
    pub lead_in_length: f64,
    pub tab_width: f64,
    pub tab_count: usize,
    pub default_kerf: f64, // Laser beam kerf compensation in mm (0.0 = disabled)
}

impl Default for CutOptimizer {
    fn default() -> Self {
        Self {
            inner_cuts_first: true,
            minimize_travel: true,
            allow_reversal: true,
            lead_in_length: 0.0,
            tab_width: 0.0,
            tab_count: 0,
            default_kerf: 0.0,
        }
    }
}

impl CutOptimizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Optimizes all shapes in a document into an ordered sequence of laser toolpaths
    pub fn optimize(&self, doc: &Document) -> OptimizedJob {
        // Step 1: Collect all polylines grouped by layer
        let mut layer_groups: std::collections::BTreeMap<i32, Vec<(usize, Polyline)>> =
            std::collections::BTreeMap::new();

        for shape in &doc.shapes {
            let layer_idx = shape.layer_index();

            // Skip Tool layers (T01, T02) and any layer with output disabled
            if let Some(layer) = doc.layers.get(&layer_idx) {
                if !layer.output || layer.is_tool() {
                    continue;
                }
            } else if layer_idx >= 30 {
                continue;
            }

            let priority = doc
                .layers
                .get(&layer_idx)
                .map(|l| l.priority)
                .unwrap_or(layer_idx as i32);

            let layer_cut_setting = doc.layers.get(&layer_idx).map(|l| &l.cut_setting);
            let polys = if let Some(cs) = layer_cut_setting {
                if cs.mode == crate::document::CutMode::Image {
                    let interval = if cs.line_interval > 0.005 {
                        cs.line_interval
                    } else {
                        0.08
                    };
                    shape.to_raster_polylines(cs.dither_mode, interval)
                } else {
                    shape.to_polylines()
                }
            } else {
                shape.to_polylines()
            };
            for p in polys {
                layer_groups.entry(priority).or_default().push((layer_idx, p));
            }
        }

        let mut final_paths: Vec<OptimizedPath> = Vec::new();
        let mut current_laser_pos = Point2D::ZERO;

        // Step 2: Process each layer by priority order
        for (_prio, items) in layer_groups {
            if items.is_empty() {
                continue;
            }

            // Step 2a: Classify inner vs outer contours
            let mut classified: Vec<OptimizedPath> = Vec::with_capacity(items.len());
            for i in 0..items.len() {
                let is_inner = if self.inner_cuts_first && items[i].1.is_closed {
                    // Check if items[i] is contained inside any other closed polygon in the layer
                    let mut contained = false;
                    if let Some(test_pt) = items[i].1.points.first() {
                        for (j, item) in items.iter().enumerate() {
                            if i != j && item.1.is_closed && item.1.contains_point(test_pt) {
                                contained = true;
                                break;
                            }
                        }
                    }
                    contained
                } else {
                    false
                };

                let layer_opt = doc.layers.get(&items[i].0);
                let is_vector_line = layer_opt.map_or(true, |l| l.cut_setting.mode == crate::document::CutMode::Line);
                let layer_kerf = layer_opt.map(|l| l.cut_setting.kerf).unwrap_or(0.0);
                let eff_kerf = if layer_kerf != 0.0 { layer_kerf } else { self.default_kerf };

                let poly = if is_vector_line && eff_kerf.abs() > 1e-5 && items[i].1.is_closed {
                    // Outer cut: laser burns on cutline, so expand shape outward (+eff_kerf / 2.0)
                    // Inner cut (hole/slot): laser burns on cutline, so shrink hole inward (-eff_kerf / 2.0)
                    let offset_dist = if is_inner { -eff_kerf * 0.5 } else { eff_kerf * 0.5 };
                    items[i].1.offset(offset_dist)
                } else {
                    items[i].1.clone()
                };

                classified.push(OptimizedPath {
                    layer_index: items[i].0,
                    polyline: poly,
                    is_inner_cut: is_inner,
                });
            }

            // Step 2b: Sort so inner cuts come before outer cuts
            if self.inner_cuts_first {
                classified.sort_by(|a, b| {
                    match (a.is_inner_cut, b.is_inner_cut) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => std::cmp::Ordering::Equal,
                    }
                });
            }

            // For Image (raster) layers, bypass TSP nearest-neighbor reordering to preserve
            // the optimal serpentine scanline order and avoid O(N^2) processing delay
            let is_image_layer = items
                .first()
                .and_then(|(l_idx, _)| doc.layers.get(l_idx))
                .map_or(false, |l| l.cut_setting.mode == crate::document::CutMode::Image);

            // Step 2c: Nearest-Neighbor TSP optimization for rapid travel (vector layers only)
            if !is_image_layer && self.minimize_travel && classified.len() > 1 {
                let mut remaining = classified;
                let mut reordered: Vec<OptimizedPath> = Vec::with_capacity(remaining.len());

                while !remaining.is_empty() {
                    let mut best_idx = 0;
                    let mut best_dist_sq = f64::INFINITY;
                    let mut should_reverse = false;

                    // When inner_cuts_first is enabled, only consider inner cuts until all inner cuts are cut
                    let has_inner = self.inner_cuts_first && remaining.iter().any(|p| p.is_inner_cut);

                    for (idx, item) in remaining.iter().enumerate() {
                        if has_inner && !item.is_inner_cut {
                            continue;
                        }

                        let pts = &item.polyline.points;
                        if pts.is_empty() {
                            continue;
                        }

                        let start_dist = current_laser_pos.distance_squared_to(&pts[0]);
                        if start_dist < best_dist_sq {
                            best_dist_sq = start_dist;
                            best_idx = idx;
                            should_reverse = false;
                        }

                        // Open paths can be reversed to save rapid travel
                        if self.allow_reversal && !item.polyline.is_closed {
                            let end_dist = current_laser_pos.distance_squared_to(pts.last().unwrap());
                            if end_dist < best_dist_sq {
                                best_dist_sq = end_dist;
                                best_idx = idx;
                                should_reverse = true;
                            }
                        }
                    }

                    let mut chosen = remaining.remove(best_idx);
                    if should_reverse {
                        chosen.polyline.reverse();
                    }

                    if let Some(last_pt) = chosen.polyline.points.last() {
                        current_laser_pos = *last_pt;
                    }
                    reordered.push(chosen);
                }
                classified = reordered;
            }

            // Step 2d: Insert Tabs / Bridges if configured
            if self.tab_width > 0.0 && self.tab_count > 0 {
                let mut tabbed_paths = Vec::new();
                for path in classified {
                    if path.polyline.is_closed && path.polyline.length() > self.tab_width * (self.tab_count as f64 + 1.0) {
                        let segments = self.insert_tabs(&path.polyline, self.tab_width, self.tab_count);
                        for seg in segments {
                            tabbed_paths.push(OptimizedPath {
                                layer_index: path.layer_index,
                                polyline: seg,
                                is_inner_cut: path.is_inner_cut,
                            });
                        }
                    } else {
                        tabbed_paths.push(path);
                    }
                }
                classified = tabbed_paths;
            }

            final_paths.extend(classified);
        }

        OptimizedJob { paths: final_paths }
    }

    /// Splits a closed polyline by injecting tab gaps of specified width
    fn insert_tabs(&self, poly: &Polyline, tab_width: f64, tab_count: usize) -> Vec<Polyline> {
        let total_len = poly.length();
        if total_len <= tab_width * 2.0 || tab_count == 0 {
            return vec![poly.clone()];
        }

        let interval = total_len / tab_count as f64;
        let mut segments = Vec::new();
        let mut current_segment = Vec::new();

        let mut accumulated_dist = 0.0;
        let mut next_tab_start = interval - (tab_width * 0.5);

        for i in 0..poly.points.len() {
            let p1 = poly.points[i];
            let p2 = poly.points[(i + 1) % poly.points.len()];
            let seg_len = p1.distance_to(&p2);

            current_segment.push(p1);

            if accumulated_dist + seg_len >= next_tab_start {
                // Tab starts on this line segment
                let t_start = ((next_tab_start - accumulated_dist) / seg_len).clamp(0.0, 1.0);
                let tab_pt1 = p1.lerp(&p2, t_start);
                current_segment.push(tab_pt1);
                segments.push(Polyline::new(current_segment, false));

                // Skip the tab gap
                let tab_end_dist = next_tab_start + tab_width;
                let t_end = ((tab_end_dist - accumulated_dist) / seg_len).clamp(0.0, 1.0);
                let tab_pt2 = p1.lerp(&p2, t_end);

                current_segment = vec![tab_pt2];
                next_tab_start += interval;
            }

            accumulated_dist += seg_len;
        }

        if !current_segment.is_empty() {
            segments.push(Polyline::new(current_segment, false));
        }

        segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Document, Shape};

    #[test]
    fn test_kerf_compensation_outer_and_inner() {
        let mut doc = Document::new("test", 100.0, 100.0);

        // Outer box: 20x20 from (0,0) to (20,20)
        let outer_pts = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(20.0, 0.0),
            Point2D::new(20.0, 20.0),
            Point2D::new(0.0, 20.0),
        ];
        doc.shapes.push(Shape::Path {
            polyline: Polyline::new(outer_pts, true),
            layer_index: 0,
        });

        // Inner hole: 10x10 from (5,5) to (15,15)
        let inner_pts = vec![
            Point2D::new(5.0, 5.0),
            Point2D::new(15.0, 5.0),
            Point2D::new(15.0, 15.0),
            Point2D::new(5.0, 15.0),
        ];
        doc.shapes.push(Shape::Path {
            polyline: Polyline::new(inner_pts, true),
            layer_index: 0,
        });

        let mut opt = CutOptimizer::new();
        opt.inner_cuts_first = true;
        opt.default_kerf = 0.20; // 0.20 mm beam diameter -> 0.10 mm offset

        let job = opt.optimize(&doc);
        assert_eq!(job.paths.len(), 2);

        // Path 0 must be inner cut
        assert!(job.paths[0].is_inner_cut);
        let inner_bounds = job.paths[0].polyline.bounds();
        // Inner hole was 10x10, shrunk by 0.1mm on each side -> 9.80 mm
        assert!((inner_bounds.width() - 9.80).abs() < 1e-4);
        assert!((inner_bounds.height() - 9.80).abs() < 1e-4);
        assert!((inner_bounds.min_x - 5.10).abs() < 1e-4);
        assert!((inner_bounds.min_y - 5.10).abs() < 1e-4);

        // Path 1 must be outer cut
        assert!(!job.paths[1].is_inner_cut);
        let outer_bounds = job.paths[1].polyline.bounds();
        // Outer box was 20x20, expanded by 0.1mm on each side -> 20.20 mm
        assert!((outer_bounds.width() - 20.20).abs() < 1e-4);
        assert!((outer_bounds.height() - 20.20).abs() < 1e-4);
        assert!((outer_bounds.min_x - (-0.10)).abs() < 1e-4);
        assert!((outer_bounds.min_y - (-0.10)).abs() < 1e-4);
    }

    #[test]
    fn test_tool_layers_and_non_output_layers_are_skipped() {
        let mut doc = Document::new("test", 100.0, 100.0);

        // Shape on Layer 0 (normal cut)
        doc.shapes.push(Shape::Rect {
            layer_index: 0,
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            corner_radius: 0.0,
        });

        // Shape on Layer 30 (T01 Tool layer, guide frame)
        doc.shapes.push(Shape::Rect {
            layer_index: 30,
            x: -5.0,
            y: -5.0,
            width: 20.0,
            height: 20.0,
            corner_radius: 0.0,
        });

        // Shape on Layer 1 with output disabled
        if let Some(l1) = doc.layers.get_mut(&1) {
            l1.output = false;
        }
        doc.shapes.push(Shape::Rect {
            layer_index: 1,
            x: 2.0,
            y: 2.0,
            width: 5.0,
            height: 5.0,
            corner_radius: 0.0,
        });

        let opt = CutOptimizer::new();
        let job = opt.optimize(&doc);

        // Only Layer 0 should be cut, Layer 30 (Tool) and Layer 1 (Output=false) must be omitted!
        assert_eq!(job.paths.len(), 1);
        assert_eq!(job.paths[0].layer_index, 0);
    }
}
