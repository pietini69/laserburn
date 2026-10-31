use std::path::Path;

/// A 3D triangle with 3 vertices in Euclidean space
#[derive(Debug, Clone, Copy)]
pub struct Triangle3D {
    pub v0: [f32; 3],
    pub v1: [f32; 3],
    pub v2: [f32; 3],
}

/// Axis along which the 3D model is projected into a 2D depth map
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionAxis {
    /// Automatically selects the thinnest axis as depth (ideal for coins, medallions, bas-reliefs)
    Auto,
    /// View looking down the Z axis (XY plane)
    Z,
    /// View looking down the Y axis (XZ plane)
    Y,
    /// View looking down the X axis (YZ plane)
    X,
}

/// 3D Mesh container for GLB / glTF and STL models
pub struct Mesh3D {
    pub triangles: Vec<Triangle3D>,
}

impl Mesh3D {
    pub fn new() -> Self {
        Self {
            triangles: Vec::new(),
        }
    }

    /// Loads a 3D model from a file based on extension (.glb, .gltf, .stl)
    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let p = path.as_ref();
        let ext = p
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "glb" | "gltf" => Self::load_glb(p),
            "stl" => Self::load_stl(p),
            _ => Err(format!(
                "Unsupported 3D mesh format: .{} (expected .glb, .gltf, or .stl)",
                ext
            )),
        }
    }

    /// Loads a glTF or binary GLB model without failing on unknown texture/material extensions (e.g. WebP)
    pub fn load_glb<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let p = path.as_ref();
        let bytes = std::fs::read(p).map_err(|e| format!("Failed to read file {}: {}", p.display(), e))?;

        // 1. If it's a binary GLB container (magic "glTF"), parse geometry directly without validating texture extensions
        if bytes.starts_with(b"glTF") {
            if let Ok(glb) = gltf::binary::Glb::from_slice(&bytes) {
                if let Ok(root) = gltf::json::Root::from_slice(&glb.json) {
                    let doc = gltf::Document::from_json_without_validation(root);
                    let bin = glb.bin.unwrap_or_default();
                    let mut triangles = Vec::new();

                    for mesh in doc.meshes() {
                        for primitive in mesh.primitives() {
                            let reader = primitive.reader(|b| if b.index() == 0 { Some(bin.as_ref()) } else { None });
                            let positions: Vec<[f32; 3]> = match reader.read_positions() {
                                Some(iter) => iter.collect(),
                                None => continue,
                            };

                            if positions.is_empty() {
                                continue;
                            }

                            if let Some(indices) = reader.read_indices() {
                                let idx_vec: Vec<u32> = indices.into_u32().collect();
                                match primitive.mode() {
                                    gltf::mesh::Mode::Triangles => {
                                        for chunk in idx_vec.chunks_exact(3) {
                                            let i0 = chunk[0] as usize;
                                            let i1 = chunk[1] as usize;
                                            let i2 = chunk[2] as usize;
                                            if i0 < positions.len() && i1 < positions.len() && i2 < positions.len() {
                                                triangles.push(Triangle3D {
                                                    v0: positions[i0],
                                                    v1: positions[i1],
                                                    v2: positions[i2],
                                                });
                                            }
                                        }
                                    }
                                    gltf::mesh::Mode::TriangleStrip => {
                                        for (strip_idx, window) in idx_vec.windows(3).enumerate() {
                                            let (i0, i1, i2) = if strip_idx % 2 == 0 {
                                                (window[0] as usize, window[1] as usize, window[2] as usize)
                                            } else {
                                                (window[1] as usize, window[0] as usize, window[2] as usize)
                                            };
                                            if i0 < positions.len() && i1 < positions.len() && i2 < positions.len() {
                                                triangles.push(Triangle3D {
                                                    v0: positions[i0],
                                                    v1: positions[i1],
                                                    v2: positions[i2],
                                                });
                                            }
                                        }
                                    }
                                    gltf::mesh::Mode::TriangleFan => {
                                        if !idx_vec.is_empty() {
                                            let center_idx = idx_vec[0] as usize;
                                            for window in idx_vec[1..].windows(2) {
                                                let i1 = window[0] as usize;
                                                let i2 = window[1] as usize;
                                                if center_idx < positions.len() && i1 < positions.len() && i2 < positions.len() {
                                                    triangles.push(Triangle3D {
                                                        v0: positions[center_idx],
                                                        v1: positions[i1],
                                                        v2: positions[i2],
                                                    });
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            } else {
                                for chunk in positions.chunks_exact(3) {
                                    triangles.push(Triangle3D {
                                        v0: chunk[0],
                                        v1: chunk[1],
                                        v2: chunk[2],
                                    });
                                }
                            }
                        }
                    }

                    if !triangles.is_empty() {
                        return Ok(Self { triangles });
                    }
                }
            }
        }

        // 2. Fallback to standard gltf::import for ASCII .gltf with external files
        let (doc, buffers, _images) =
            gltf::import(p).map_err(|e| format!("Failed to parse GLB/glTF {}: {}", p.display(), e))?;

        let mut triangles = Vec::new();

        for mesh in doc.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|b| Some(&buffers[b.index()]));
                let positions: Vec<[f32; 3]> = match reader.read_positions() {
                    Some(iter) => iter.collect(),
                    None => continue,
                };

                if positions.is_empty() {
                    continue;
                }

                if let Some(indices) = reader.read_indices() {
                    let idx_vec: Vec<u32> = indices.into_u32().collect();
                    match primitive.mode() {
                        gltf::mesh::Mode::Triangles => {
                            for chunk in idx_vec.chunks_exact(3) {
                                let i0 = chunk[0] as usize;
                                let i1 = chunk[1] as usize;
                                let i2 = chunk[2] as usize;
                                if i0 < positions.len() && i1 < positions.len() && i2 < positions.len() {
                                    triangles.push(Triangle3D {
                                        v0: positions[i0],
                                        v1: positions[i1],
                                        v2: positions[i2],
                                    });
                                }
                            }
                        }
                        gltf::mesh::Mode::TriangleStrip => {
                            for (strip_idx, window) in idx_vec.windows(3).enumerate() {
                                let (i0, i1, i2) = if strip_idx % 2 == 0 {
                                    (window[0] as usize, window[1] as usize, window[2] as usize)
                                } else {
                                    (window[1] as usize, window[0] as usize, window[2] as usize)
                                };
                                if i0 < positions.len() && i1 < positions.len() && i2 < positions.len() {
                                    triangles.push(Triangle3D {
                                        v0: positions[i0],
                                        v1: positions[i1],
                                        v2: positions[i2],
                                    });
                                }
                            }
                        }
                        gltf::mesh::Mode::TriangleFan => {
                            if !idx_vec.is_empty() {
                                let center_idx = idx_vec[0] as usize;
                                for window in idx_vec[1..].windows(2) {
                                    let i1 = window[0] as usize;
                                    let i2 = window[1] as usize;
                                    if center_idx < positions.len() && i1 < positions.len() && i2 < positions.len() {
                                        triangles.push(Triangle3D {
                                            v0: positions[center_idx],
                                            v1: positions[i1],
                                            v2: positions[i2],
                                        });
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                } else {
                    for chunk in positions.chunks_exact(3) {
                        triangles.push(Triangle3D {
                            v0: chunk[0],
                            v1: chunk[1],
                            v2: chunk[2],
                        });
                    }
                }
            }
        }

        if triangles.is_empty() {
            return Err("No 3D triangle geometry found in GLB/glTF model".to_string());
        }

        Ok(Self { triangles })
    }

    /// Loads an STL model (supporting both binary and ASCII STL formats)
    pub fn load_stl<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let p = path.as_ref();
        let bytes = std::fs::read(p).map_err(|e| format!("Failed to read STL {}: {}", p.display(), e))?;

        if bytes.len() < 84 {
            return Err("STL file too short".to_string());
        }

        // Check if binary STL: total bytes == 80 (header) + 4 (num_triangles) + num_triangles * 50
        let num_triangles = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]) as usize;
        let expected_binary_len = 84 + num_triangles * 50;

        if bytes.len() == expected_binary_len {
            let mut triangles = Vec::with_capacity(num_triangles);
            let mut offset = 84;
            for _ in 0..num_triangles {
                if offset + 50 > bytes.len() {
                    break;
                }
                // Skip normal (12 bytes)
                let v0x = f32::from_le_bytes([bytes[offset + 12], bytes[offset + 13], bytes[offset + 14], bytes[offset + 15]]);
                let v0y = f32::from_le_bytes([bytes[offset + 16], bytes[offset + 17], bytes[offset + 18], bytes[offset + 19]]);
                let v0z = f32::from_le_bytes([bytes[offset + 20], bytes[offset + 21], bytes[offset + 22], bytes[offset + 23]]);

                let v1x = f32::from_le_bytes([bytes[offset + 24], bytes[offset + 25], bytes[offset + 26], bytes[offset + 27]]);
                let v1y = f32::from_le_bytes([bytes[offset + 28], bytes[offset + 29], bytes[offset + 30], bytes[offset + 31]]);
                let v1z = f32::from_le_bytes([bytes[offset + 32], bytes[offset + 33], bytes[offset + 34], bytes[offset + 35]]);

                let v2x = f32::from_le_bytes([bytes[offset + 36], bytes[offset + 37], bytes[offset + 38], bytes[offset + 39]]);
                let v2y = f32::from_le_bytes([bytes[offset + 40], bytes[offset + 41], bytes[offset + 42], bytes[offset + 43]]);
                let v2z = f32::from_le_bytes([bytes[offset + 44], bytes[offset + 45], bytes[offset + 46], bytes[offset + 47]]);

                triangles.push(Triangle3D {
                    v0: [v0x, v0y, v0z],
                    v1: [v1x, v1y, v1z],
                    v2: [v2x, v2y, v2z],
                });
                offset += 50;
            }
            return Ok(Self { triangles });
        }

        // Fallback: parse ASCII STL
        let text = String::from_utf8_lossy(&bytes);
        let mut triangles = Vec::new();
        let mut current_verts = Vec::with_capacity(3);

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("vertex") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 4 {
                    if let (Ok(x), Ok(y), Ok(z)) = (parts[1].parse::<f32>(), parts[2].parse::<f32>(), parts[3].parse::<f32>()) {
                        current_verts.push([x, y, z]);
                        if current_verts.len() == 3 {
                            triangles.push(Triangle3D {
                                v0: current_verts[0],
                                v1: current_verts[1],
                                v2: current_verts[2],
                            });
                            current_verts.clear();
                        }
                    }
                }
            }
        }

        if triangles.is_empty() {
            return Err("Failed to parse valid 3D triangles from STL file".to_string());
        }

        Ok(Self { triangles })
    }

    /// Computes the 3D axis-aligned bounding box (min_xyz, max_xyz)
    pub fn bounds(&self) -> ([f32; 3], [f32; 3]) {
        let mut min_pt = [f32::INFINITY; 3];
        let mut max_pt = [f32::NEG_INFINITY; 3];

        for t in &self.triangles {
            for v in &[t.v0, t.v1, t.v2] {
                for i in 0..3 {
                    if v[i] < min_pt[i] {
                        min_pt[i] = v[i];
                    }
                    if v[i] > max_pt[i] {
                        max_pt[i] = v[i];
                    }
                }
            }
        }

        (min_pt, max_pt)
    }

    /// Renders an orthographic top-down depth map (Z-buffer heightmap) into an RGBA image.
    ///
    /// Pixels where the model exists have alpha = 255 with grayscale intensity representing relief height:
    /// - High surface (peaks, coin details) = light gray (250) -> minimal laser burn
    /// - Deep floor (recesses, base) = dark gray (10) -> deep laser burn
    /// - Background outside the model = pure white (255) with alpha = 0 -> laser leaves untouched!
    pub fn render_heightmap(
        &self,
        resolution: usize,
        axis: ProjectionAxis,
    ) -> Result<(Vec<u8>, usize, usize), String> {
        if self.triangles.is_empty() {
            return Err("Mesh contains no triangles".to_string());
        }

        let (min_pt, max_pt) = self.bounds();
        let dx = (max_pt[0] - min_pt[0]).abs();
        let dy = (max_pt[1] - min_pt[1]).abs();
        let dz = (max_pt[2] - min_pt[2]).abs();

        // Determine projection mapping: (u_idx, v_idx, depth_idx)
        let (u_idx, v_idx, depth_idx) = match axis {
            ProjectionAxis::Auto => {
                // Auto-detect relief axis: the axis with the smallest extent is the thickness/depth axis!
                if dz <= dx && dz <= dy {
                    (0, 1, 2) // Depth is Z
                } else if dy <= dx && dy <= dz {
                    (0, 2, 1) // Depth is Y
                } else {
                    (1, 2, 0) // Depth is X
                }
            }
            ProjectionAxis::Z => (0, 1, 2),
            ProjectionAxis::Y => (0, 2, 1),
            ProjectionAxis::X => (1, 2, 0),
        };

        let span_u = (max_pt[u_idx] - min_pt[u_idx]).abs().max(1e-4);
        let span_v = (max_pt[v_idx] - min_pt[v_idx]).abs().max(1e-4);
        let max_span = span_u.max(span_v);

        let target_res = resolution.clamp(64, 4096);
        let w_px = ((span_u / max_span) * target_res as f32).round().clamp(16.0, 4096.0) as usize;
        let h_px = ((span_v / max_span) * target_res as f32).round().clamp(16.0, 4096.0) as usize;

        let mut depth_buffer = vec![f32::NEG_INFINITY; w_px * h_px];

        let min_u = min_pt[u_idx];
        let min_v = min_pt[v_idx];

        let u_to_px = (w_px as f32 - 1.0) / span_u;
        let v_to_py = (h_px as f32 - 1.0) / span_v;

        // Software orthographic triangle rasterizer (Z-buffer)
        for tri in &self.triangles {
            let p0 = [
                (tri.v0[u_idx] - min_u) * u_to_px,
                (span_v - (tri.v0[v_idx] - min_v)) * v_to_py, // Invert V so top of 3D model is top of image
                tri.v0[depth_idx],
            ];
            let p1 = [
                (tri.v1[u_idx] - min_u) * u_to_px,
                (span_v - (tri.v1[v_idx] - min_v)) * v_to_py,
                tri.v1[depth_idx],
            ];
            let p2 = [
                (tri.v2[u_idx] - min_u) * u_to_px,
                (span_v - (tri.v2[v_idx] - min_v)) * v_to_py,
                tri.v2[depth_idx],
            ];

            // Bounding box in pixel coordinates
            let min_x = (p0[0].min(p1[0]).min(p2[0]).floor() as isize).clamp(0, w_px as isize - 1) as usize;
            let max_x = (p0[0].max(p1[0]).max(p2[0]).ceil() as isize).clamp(0, w_px as isize - 1) as usize;
            let min_y = (p0[1].min(p1[1]).min(p2[1]).floor() as isize).clamp(0, h_px as isize - 1) as usize;
            let max_y = (p0[1].max(p1[1]).max(p2[1]).ceil() as isize).clamp(0, h_px as isize - 1) as usize;

            if min_x > max_x || min_y > max_y {
                continue;
            }

            // Signed 2D triangle area denominator
            let denom = (p1[1] - p2[1]) * (p0[0] - p2[0]) + (p2[0] - p1[0]) * (p0[1] - p2[1]);
            if denom.abs() < 1e-6 {
                continue; // Degenerate triangle
            }
            let inv_denom = 1.0 / denom;

            for py in min_y..=max_y {
                let sample_y = py as f32 + 0.5;
                let row_offset = py * w_px;

                for px in min_x..=max_x {
                    let sample_x = px as f32 + 0.5;

                    let w0 = ((p1[1] - p2[1]) * (sample_x - p2[0]) + (p2[0] - p1[0]) * (sample_y - p2[1])) * inv_denom;
                    let w1 = ((p2[1] - p0[1]) * (sample_x - p2[0]) + (p0[0] - p2[0]) * (sample_y - p2[1])) * inv_denom;
                    let w2 = 1.0 - w0 - w1;

                    // Inside triangle test
                    if w0 >= -1e-4 && w1 >= -1e-4 && w2 >= -1e-4 {
                        let z_interp = w0 * p0[2] + w1 * p1[2] + w2 * p2[2];
                        let idx = row_offset + px;
                        if z_interp > depth_buffer[idx] {
                            depth_buffer[idx] = z_interp;
                        }
                    }
                }
            }
        }

        // Find min and max depth of hit pixels
        let mut z_min = f32::INFINITY;
        let mut z_max = f32::NEG_INFINITY;
        let mut hit_count = 0;

        for &d in &depth_buffer {
            if d > f32::NEG_INFINITY {
                hit_count += 1;
                if d < z_min {
                    z_min = d;
                }
                if d > z_max {
                    z_max = d;
                }
            }
        }

        if hit_count == 0 {
            return Err("Rasterization produced 0 pixels: check mesh coordinates".to_string());
        }

        let depth_range = (z_max - z_min).max(1e-5);
        let mut rgba = Vec::with_capacity(w_px * h_px * 4);

        for &d in &depth_buffer {
            if d == f32::NEG_INFINITY {
                // Background outside 3D model -> Pure white with 0 alpha (laser skips)
                rgba.push(255);
                rgba.push(255);
                rgba.push(255);
                rgba.push(0);
            } else {
                // Normalized depth in [0.0, 1.0]:
                // 1.0 = top surface / peaks (minimal burn)
                // 0.0 = deep valleys / base (deepest burn)
                let norm = ((d - z_min) / depth_range).clamp(0.0, 1.0);
                // Map to 10..250 grayscale
                let val = (10.0 + norm * 240.0).round().clamp(0.0, 255.0) as u8;
                rgba.push(val);
                rgba.push(val);
                rgba.push(val);
                rgba.push(255);
            }
        }

        Ok((rgba, w_px, h_px))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthetic_wedge_mesh_rasterization() {
        let mut mesh = Mesh3D::new();
        // Create a sloped wedge: base on Z=0, peak on Z=10
        // Triangle 1: (0,0,0), (10,0,0), (5,10,10)
        mesh.triangles.push(Triangle3D {
            v0: [0.0, 0.0, 0.0],
            v1: [10.0, 0.0, 0.0],
            v2: [5.0, 10.0, 10.0],
        });

        let (rgba, w, h) = mesh.render_heightmap(128, ProjectionAxis::Auto).unwrap();
        assert!(w > 0 && h > 0);
        assert_eq!(rgba.len(), w * h * 4);

        // Verify that there are non-transparent pixels (model) and transparent pixels (background)
        let model_pixels = rgba.chunks_exact(4).filter(|p| p[3] == 255).count();
        let bg_pixels = rgba.chunks_exact(4).filter(|p| p[3] == 0).count();
        assert!(model_pixels > 0);
        assert!(bg_pixels > 0);
    }

    #[test]
    fn test_ascii_stl_parsing() {
        let ascii_stl = r#"solid test
  facet normal 0 0 1
    outer loop
      vertex 0.0 0.0 5.0
      vertex 10.0 0.0 5.0
      vertex 5.0 10.0 10.0
    endloop
  endfacet
endsolid test
"#;
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_triangle.stl");
        std::fs::write(&file_path, ascii_stl).unwrap();

        let mesh = Mesh3D::load_stl(&file_path).unwrap();
        assert_eq!(mesh.triangles.len(), 1);
        let bounds = mesh.bounds();
        assert_eq!(bounds.0[0], 0.0);
        assert_eq!(bounds.1[0], 10.0);

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_load_real_glb_duck() {
        let coin_path = "/Volumes/Jättitila/Laser/aberrantrealities-gold-5589.glb";
        if std::path::Path::new(coin_path).exists() {
            let mesh = Mesh3D::load_glb(coin_path).expect("Failed to load coin GLB");
            assert!(!mesh.triangles.is_empty());
            println!("Loaded {} triangles from coin!", mesh.triangles.len());
            let (rgba, w, h) = mesh.render_heightmap(1024, ProjectionAxis::Auto).unwrap();
            assert!(w > 0 && h > 0);
            assert_eq!(rgba.len(), w * h * 4);
        } else {
            let duck_path = "/Volumes/Dataa/pinokio/drive/drives/pip/gradio/5.4.0/lib/python3.10/site-packages/gradio/templates/frontend/static/img/Duck.glb";
            if std::path::Path::new(duck_path).exists() {
                let mesh = Mesh3D::load_glb(duck_path).unwrap();
                assert!(!mesh.triangles.is_empty());
            }
        }
    }
}

