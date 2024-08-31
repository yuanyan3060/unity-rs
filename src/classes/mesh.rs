#![allow(non_upper_case_globals)]
use super::animation_clip::{AnimationClip, PackedFloatVector, PackedIntVector};
use super::texture2d::StreamingInfo;
use super::FromObject;
use crate::error::{UnityError, UnityResult};
use crate::math::{Matrix4x4, Vector3};
use crate::object::ObjectInfo;
use crate::reader::{ByteOrder, Reader};
use crate::Object;
use num_enum::TryFromPrimitive;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone, Copy, Default)]
#[repr(i32)]
pub enum GfxPrimitiveType {
    #[default]
    Triangles = 0,
    TriangleStrip = 1,
    Quads = 2,
    Lines = 3,
    LineStrip = 4,
    Points = 5,
}

#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub use_16_bit_indices: bool,
    pub sub_meshes: Vec<SubMesh>,
    pub index_buffer: Vec<u32>,
    pub shapes: Option<BlendShapeData>,
    pub bind_pose: Vec<Matrix4x4>,
    pub bone_name_hashes: Vec<u32>,
    pub vertex_count: usize,
    pub vertices: Vec<f32>,
    pub skin: Option<Vec<BoneWeights4>>,
    pub normals: Vec<f32>,
    pub colors: Vec<f32>,
    pub uv0: Vec<f32>,
    pub uv1: Vec<f32>,
    pub uv2: Vec<f32>,
    pub uv3: Vec<f32>,
    pub uv4: Vec<f32>,
    pub uv5: Vec<f32>,
    pub uv6: Vec<f32>,
    pub uv7: Vec<f32>,
    pub tangents: Vec<f32>,
    pub vertex_data: Option<VertexData>,
    pub compressed_mesh: Option<CompressedMesh>,
    pub stream_data: Option<StreamingInfo>,
    pub indices: Vec<u32>,
}

impl Mesh {
    fn process_data(&mut self, object: &Object) -> UnityResult<()> {
        let version = object.info.version;
        if let (Some(stream), Some(vertex_data)) = (&self.stream_data, self.vertex_data.as_mut()) {
            if !stream.path.is_empty() && vertex_data.vertex_count > 0 {
                let path = stream.path.split('/').last().ok_or(UnityError::InvalidValue)?;
                for i in 0..object.bundle.nodes.len() {
                    let node = &object.bundle.nodes[i];
                    if node.path != path {
                        continue;
                    }
                    let file = &object.bundle.files[i];
                    let mut r = Reader::new(file.as_slice(), ByteOrder::Big);
                    r.set_offset(stream.offset as usize)?;
                    vertex_data.data_size = r.read_u8_list(stream.size as usize)?;
                }
            }
        }
        if version[0] > 3 || (version[0] == 3 && version[1] >= 5) {
            self.read_vertex_data(object)?;
        }
        if version[0] > 2 || (version[0] == 2 && version[1] >= 6) {
            self.decompress_compressed_mesh(object)?;
        }
        self.get_triangles(object)?;
        Ok(())
    }

    fn read_vertex_data(&mut self, object: &Object) -> UnityResult<()> {
        let version = object.info.version;
        let Some(vertex_data) = self.vertex_data.as_mut() else {
            return Ok(());
        };
        self.vertex_count = vertex_data.vertex_count as usize;
        let vertex_count = vertex_data.vertex_count as usize;
        for (chn, channel) in vertex_data.channels.iter_mut().enumerate() {
            if channel.dimension == 0 {
                continue;
            }
            let Some(stream) = vertex_data.streams.get(channel.stream as usize) else {
                continue;
            };
            if (stream.channel_mask >> chn) & 0x1 == 0 {
                continue;
            }
            if version[0] < 2018 && chn == 2 && channel.format == 2 {
                channel.dimension = 4;
            }
            let vertex_format = VertexFormat::load(channel.format, version)?;
            let component_byte_size = vertex_format.get_format_size() as usize;
            let mut component_bytes = Vec::with_capacity(vertex_count * channel.dimension as usize * component_byte_size);
            for v in 0..vertex_count {
                let vertex_offset = stream.offset as usize + channel.offset as usize + stream.stride as usize * v;
                for d in 0..channel.dimension {
                    let component_offset = vertex_offset + component_byte_size * d as usize;
                    let end = component_offset + component_byte_size;
                    let sub = vertex_data.data_size.get(component_offset..end).ok_or_else(|| UnityError::Eof)?;
                    component_bytes.extend_from_slice(sub);
                }
            }
            if object.info.bytes_order == ByteOrder::Big && component_byte_size > 1 {
                component_bytes.chunks_mut(component_byte_size).for_each(|x| x.reverse());
            }
            let mut components_int_array = Vec::new();
            let mut components_f32_array = Vec::new();
            if vertex_format.is_int() {
                components_int_array = bytes_to_i32_vec(&component_bytes, vertex_format);
            } else {
                components_f32_array = bytes_to_f32_vec(&component_bytes, vertex_format);
            }
            if version[0] >= 2018 {
                match chn {
                    0 => self.vertices = components_f32_array,
                    1 => self.normals = components_f32_array,
                    2 => self.tangents = components_f32_array,
                    3 => self.colors = components_f32_array,
                    4 => self.uv0 = components_f32_array,
                    5 => self.uv1 = components_f32_array,
                    6 => self.uv2 = components_f32_array,
                    7 => self.uv3 = components_f32_array,
                    8 => self.uv4 = components_f32_array,
                    9 => self.uv5 = components_f32_array,
                    10 => self.uv6 = components_f32_array,
                    11 => self.uv7 = components_f32_array,
                    12 => {
                        if self.skin.is_none() {
                            let mut skins = vec![BoneWeights4::default(); self.vertex_count];
                            for (i, skin) in skins.iter_mut().enumerate() {
                                for j in 0..channel.dimension {
                                    let Some(value) = components_f32_array.get(i * channel.dimension as usize + j as usize).copied() else {
                                        return Err(UnityError::Eof);
                                    };
                                    let Some(weight) = skin.weight.get_mut(j as usize) else {
                                        return Err(UnityError::Eof);
                                    };
                                    *weight = value;
                                }
                            }
                            self.skin = Some(skins)
                        }
                    }
                    13 => {
                        if self.skin.is_none() {
                            let mut skins = vec![BoneWeights4::default(); self.vertex_count];
                            for (i, skin) in skins.iter_mut().enumerate() {
                                for j in 0..channel.dimension {
                                    let Some(value) = components_int_array.get(i * channel.dimension as usize + j as usize).copied() else {
                                        return Err(UnityError::Eof);
                                    };
                                    let Some(index) = skin.bone_index.get_mut(j as usize) else {
                                        return Err(UnityError::Eof);
                                    };
                                    *index = value;
                                }
                            }
                            self.skin = Some(skins)
                        }
                    }
                    _ => {}
                }
            } else {
                match chn {
                    0 => self.vertices = components_f32_array,
                    1 => self.normals = components_f32_array,
                    2 => self.colors = components_f32_array,
                    3 => self.uv0 = components_f32_array,
                    4 => self.uv1 = components_f32_array,
                    5 => {
                        if version[0] >= 5 {
                            self.uv2 = components_f32_array;
                        } else {
                            self.tangents = components_f32_array;
                        }
                    }
                    6 => {
                        self.uv3 = components_f32_array;
                    }
                    7 => {
                        self.tangents = components_f32_array;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn decompress_compressed_mesh(&mut self, object: &Object) -> UnityResult<()> {
        let version = object.info.version;
        let Some(compressed_mesh) = &mut self.compressed_mesh else {
            return Ok(());
        };
        if compressed_mesh.vertices.num_items > 0 {
            self.vertex_count = compressed_mesh.vertices.num_items as usize / 3;
            self.vertices = compressed_mesh.vertices.unpack_floats(3, 3 * 4, 0, None)
        }
        if compressed_mesh.uv.num_items > 0 {
            let uv_info = compressed_mesh.uv_info;
            if uv_info != 0 {
                const k_info_bits_per_uv: usize = 4;
                const k_uv_dimension_mask: u32 = 3;
                const k_uv_channel_exists: u32 = 4;
                const k_max_tex_coord_shader_channels: usize = 8;
                let mut uv_src_offset = 0;
                for uv in 0..k_max_tex_coord_shader_channels {
                    let mut tex_coord_bits = uv_info >> (uv * k_info_bits_per_uv);
                    tex_coord_bits &= (1 << k_info_bits_per_uv) - 1;
                    if (tex_coord_bits & k_uv_channel_exists) != 0 {
                        let uv_dim = 1 + (tex_coord_bits & k_uv_dimension_mask) as i32;
                        let uv_value = compressed_mesh.uv.unpack_floats(uv_dim as usize, uv_dim as usize * 4, uv_src_offset, Some(self.vertex_count));
                        uv_src_offset += uv_dim as usize * self.vertex_count;
                        match uv {
                            0 => self.uv0 = uv_value,
                            1 => self.uv1 = uv_value,
                            2 => self.uv2 = uv_value,
                            3 => self.uv3 = uv_value,
                            4 => self.uv4 = uv_value,
                            5 => self.uv5 = uv_value,
                            6 => self.uv6 = uv_value,
                            7 => self.uv7 = uv_value,
                            _ => return Err(UnityError::Eof),
                        }
                    }
                }
            } else {
                self.uv0 = compressed_mesh.uv.unpack_floats(2, 2 * 4, 0, Some(self.vertex_count));
                if compressed_mesh.uv.num_items as usize >= self.vertex_count * 4 {
                    self.uv1 = compressed_mesh.uv.unpack_floats(2, 2 * 4, self.vertex_count * 2, Some(self.vertex_count))
                }
            }
        }
        if version[0] < 5 {
            if let Some(bind_poses) = &compressed_mesh.bind_poses {
                let size = bind_poses.num_items as usize / 16;
                self.bind_pose = Vec::with_capacity(size);
                let unpacked = bind_poses.unpack_floats(16, 4 * 16, 0, None);
                let mut buffer = [0.0f32; 16];
                unpacked.chunks_exact(16).for_each(|x| {
                    buffer.copy_from_slice(x);
                    self.bind_pose.push(Matrix4x4::from_array(buffer))
                });
            }
        }
        if compressed_mesh.normals.num_items > 0 {
            let normal_data = compressed_mesh.normals.unpack_floats(2, 4 * 2, 0, None);
            let signs = compressed_mesh.normal_signs.unpack_ints();
            let size = compressed_mesh.normals.num_items / 2 * 3;
            self.normals = Vec::with_capacity(size as usize);
            let iter = normal_data.chunks_exact(2).map(|x| (x[0], x[1]));
            for ((mut x, mut y), sign) in iter.zip(signs) {
                let zsqr = 1.0 - x * x - y * y;
                let mut z: f32;
                if zsqr >= 0.0 {
                    z = zsqr.sqrt()
                } else {
                    z = 0.0;
                    let mut v = Vector3::new(x, y, z);
                    v.normalize();
                    x = v.x;
                    y = v.y;
                    z = v.z;
                }
                if sign == 0 {
                    z = -z;
                }
                self.normals.extend([x, y, z])
            }
        }
        if compressed_mesh.tangents.num_items > 0 {
            let tangent_data = compressed_mesh.tangents.unpack_floats(2, 4 * 2, 0, None);
            let signs = compressed_mesh.tangent_signs.unpack_ints();
            let size = compressed_mesh.tangents.num_items as usize / 2 * 4;
            self.tangents = Vec::with_capacity(size);
            let iter = tangent_data.chunks_exact(2).map(|x| (x[0], x[1]));
            let iter_sign = signs.chunks_exact(2).map(|x| (x[0], x[1]));
            for ((mut x, mut y), (sign1, sign2)) in iter.zip(iter_sign) {
                let zsqr = 1.0 - x * x - y * y;
                let mut z: f32;
                if zsqr >= 0.0 {
                    z = zsqr.sqrt()
                } else {
                    z = 0.0;
                    let mut v = Vector3::new(x, y, z);
                    v.normalize();
                    x = v.x;
                    y = v.y;
                    z = v.z;
                }
                if sign1 == 0 {
                    z = -z;
                }
                let w = if sign2 > 0 { 1.0 } else { -1.0 };
                self.tangents.extend([x, y, z, w])
            }
        }
        if version[0] >= 5 {
            if let Some(float_colors) = &compressed_mesh.float_colors {
                if float_colors.num_items > 0 {
                    self.colors = float_colors.unpack_floats(1, 4, 0, None)
                }
            }
        }
        if compressed_mesh.weights.num_items > 0 {
            let weights = compressed_mesh.weights.unpack_ints();
            let bone_indices = compressed_mesh.bone_indices.unpack_ints();
            let mut skins = vec![BoneWeights4::default(); self.vertex_count];
            let mut bone_pos = 0;
            let mut bone_index_pos = 0;
            let mut j = 0;
            let mut sum = 0;
            for weight in weights {
                let Some(bone) = skins.get_mut(bone_pos) else {
                    continue;
                };
                bone.weight[j] = weight as f32 / 31.0;
                bone.bone_index[j] = bone_indices[bone_index_pos];
                bone_index_pos += 1;
                j += 1;
                sum += weight;
                if sum >= 31 {
                    bone.bone_index = Default::default();
                    bone.weight = Default::default();
                    bone_pos += 1;
                    j = 0;
                    sum = 0;
                } else if j == 3 {
                    bone.weight[j] = (31 - sum) as f32 / 31.0;
                    bone.bone_index[j] = bone_indices[bone_index_pos];
                    bone_index_pos += 1;
                    bone_pos += 1;
                    j = 0;
                    sum = 0;
                }
            }
            self.skin = Some(skins);
        }
        if compressed_mesh.triangles.num_items > 0 {
            self.index_buffer = compressed_mesh.triangles.unpack_ints().into_iter().map(|x| x as u32).collect();
        }
        if let Some(colors) = &mut compressed_mesh.colors {
            if colors.num_items > 0 {
                colors.num_items *= 4;
                colors.bit_size /= 4;
                let temp_colors = colors.unpack_ints();
                self.colors = temp_colors.into_iter().map(|x| x as f32 / 255.0).collect()
            }
        }
        Ok(())
    }

    fn get_triangles(&mut self, object: &Object) -> UnityResult<()> {
        let version = object.info.version;
        for sub_mesh in &mut self.sub_meshes {
            let mut first_index = sub_mesh.first_bytes as usize / 2;
            if !self.use_16_bit_indices {
                first_index /= 2;
            }
            let index_count = sub_mesh.index_count as usize;
            let topology = sub_mesh.topology;
            if topology == GfxPrimitiveType::Triangles {
                let sub = self.index_buffer.get(first_index..(first_index + index_count - index_count % 3)).ok_or_else(|| UnityError::Eof)?;
                self.indices.extend_from_slice(sub)
            } else if version[0] < 4 || topology == GfxPrimitiveType::TriangleStrip {
                let mut tri_index = 0;
                let mut iter = self.index_buffer.get(first_index..).ok_or_else(|| UnityError::Eof)?.windows(3).enumerate();
                while let Some((i, &[a, b, c])) = iter.next() {
                    if a == b || a == c || b == c {
                        continue;
                    }
                    if i & 1 == 1 {
                        self.indices.push(b);
                        self.indices.push(a);
                    } else {
                        self.indices.push(a);
                        self.indices.push(b);
                    }
                    self.indices.push(c);
                    tri_index += 3;
                }
                sub_mesh.index_count = tri_index;
            } else if topology == GfxPrimitiveType::Quads {
                let mut iter = self.index_buffer.get(first_index..).ok_or_else(|| UnityError::Eof)?.windows(4);
                while let Some(&[a, b, c, d]) = iter.next() {
                    self.indices.extend([a, b, c, d, a, b, c, d])
                }
                sub_mesh.index_count = index_count as u32 / 2 * 3
            } else {
                return Err(UnityError::CustomError("Failed getting triangles. Submesh topology is lines or points.".to_string()));
            }
        }
        Ok(())
    }
}

impl<'a> FromObject<'a> for Mesh {
    fn load(object: &'a crate::Object<'a>) -> UnityResult<Self> {
        let version = object.info.version;
        let mut r = object.info.get_reader();
        let name = r.read_aligned_string()?;
        let mut ret = Mesh {
            name,
            use_16_bit_indices: false,
            sub_meshes: Vec::new(),
            index_buffer: Vec::new(),
            shapes: None,
            bind_pose: Vec::new(),
            bone_name_hashes: Vec::new(),
            vertex_count: 0,
            vertices: Vec::new(),
            skin: None,
            normals: Vec::new(),
            colors: Vec::new(),
            uv0: Vec::new(),
            uv1: Vec::new(),
            uv2: Vec::new(),
            uv3: Vec::new(),
            uv4: Vec::new(),
            uv5: Vec::new(),
            uv6: Vec::new(),
            uv7: Vec::new(),
            tangents: Vec::new(),
            vertex_data: None,
            compressed_mesh: None,
            stream_data: None,
            indices: Vec::new(),
        };
        ret.use_16_bit_indices = if version[0] < 3 || (version[0] == 3 && version[1] < 5) { r.read_i32()? > 0 } else { false };
        if version[0] == 2 && version[1] <= 5 {
            let index_buffer_size = r.read_i32()?;
            if ret.use_16_bit_indices {
                let index_buffer_size = index_buffer_size as usize / 2;
                ret.index_buffer = Vec::with_capacity(index_buffer_size);
                for _ in 0..index_buffer_size {
                    ret.index_buffer.push(r.read_u16()? as u32);
                }
            } else {
                ret.index_buffer = r.read_u32_list(index_buffer_size as usize / 4)?;
            }
        }
        let sub_meshes_size = r.read_i32()? as usize;
        ret.sub_meshes = Vec::with_capacity(sub_meshes_size);
        for _ in 0..sub_meshes_size {
            ret.sub_meshes.push(SubMesh::load(&object.info, &mut r)?)
        }
        if version[0] > 4 || (version[0] == 4 && version[1] >= 1) {
            ret.shapes = Some(BlendShapeData::load(&object.info, &mut r)?)
        };
        if version[0] > 4 || (version[0] == 4 && version[1] >= 3) {
            let size = r.read_i32()?;
            ret.bind_pose = r.read_matrix4x4_list(size as usize)?;
            let size = r.read_i32()?;
            ret.bone_name_hashes = r.read_u32_list(size as usize)?;
            let _root_bone_name_hash = r.read_u32()?;
        }

        if version[0] > 2 || (version[0] == 2 && version[1] >= 6) {
            if version[0] >= 2019 {
                let _bones_aabb_size = r.read_i32()?;
                let mut _bones_aabb = Vec::new();
                for _ in 0.._bones_aabb_size {
                    _bones_aabb.push(MinMaxAABB::load(&object.info, &mut r)?)
                }
                let _variable_bone_count_weights = r.read_u32()?;
            }
            let mesh_compression = r.read_u8()?;
            if version[0] >= 4 {
                if version[0] < 5 {
                    let _stream_compression = r.read_u8()?;
                }
                let _is_readable = r.read_bool()?;
                let _keep_vertices = r.read_bool()?;
                let _keep_indices = r.read_bool()?;
            }
            r.align(4)?;
            if (version[0] > 2017 || (version[0] == 2017 && version[1] >= 4)) || //2017.4
            ((version[0] == 2017 && version[1] == 3 && version[2] == 1) && object.info.build_type.is_patch()) || //fixed after 2017.3.1px
            ((version[0] == 2017 && version[1] == 3) && mesh_compression == 0)
            {
                let index_format = r.read_i32()?;
                ret.use_16_bit_indices = index_format == 0;
            }
            let index_buffer_size = r.read_i32()?;
            if ret.use_16_bit_indices {
                let index_buffer_size = index_buffer_size as usize / 2;
                ret.index_buffer = Vec::with_capacity(index_buffer_size);
                for _ in 0..index_buffer_size {
                    ret.index_buffer.push(r.read_u16()? as u32);
                }
                r.align(4)?;
            } else {
                ret.index_buffer = r.read_u32_list(index_buffer_size as usize / 4)?;
            }
        }
        if version[0] < 3 || (version[0] == 3 && version[1] < 5) {
            ret.vertex_count = r.read_i32()? as usize;
            ret.vertices = r.read_f32_list(ret.vertex_count)?;
            let size = r.read_i32()?;
            let mut skin = Vec::with_capacity(size as usize);
            for _ in 0..size {
                skin.push(BoneWeights4::load(&object.info, &mut r)?)
            }
            ret.skin = Some(skin);
            let size = r.read_i32()?;
            ret.bind_pose = r.read_matrix4x4_list(size as usize)?;
            let size = r.read_i32()? as usize;
            ret.uv0 = r.read_f32_list(size * 2)?;
            let size = r.read_i32()? as usize;
            ret.uv1 = r.read_f32_list(size * 2)?;
            if version[0] == 2 && version[1] <= 5 {
                let tangent_space_size = r.read_i32()? as usize;
                ret.normals = Vec::with_capacity(tangent_space_size * 3);
                ret.tangents = Vec::with_capacity(tangent_space_size * 4);
                for _ in 0..tangent_space_size {
                    for _ in 0..3 {
                        ret.normals.push(r.read_f32()?)
                    }
                    for _ in 0..4 {
                        ret.tangents.push(r.read_f32()?)
                    }
                }
            } else {
                let size = r.read_i32()? as usize;
                ret.tangents = r.read_f32_list(size * 4)?;
                let size = r.read_i32()? as usize;
                ret.normals = r.read_f32_list(size * 3)?;
            }
        } else {
            if version[0] < 2018 || (version[0] == 2018 && version[1] < 2) {
                let size = r.read_i32()?;
                let mut skin = Vec::with_capacity(size as usize);
                for _ in 0..size {
                    skin.push(BoneWeights4::load(&object.info, &mut r)?)
                }
                ret.skin = Some(skin);
            }
            if version[0] == 3 || (version[0] == 4 && version[1] <= 2) {
                let size = r.read_i32()?;
                ret.bind_pose = r.read_matrix4x4_list(size as usize)?;
            }
            ret.vertex_data = Some(VertexData::load(&object.info, &mut r)?);
        }
        if version[0] > 2 || (version[0] == 2 && version[1] >= 6) {
            ret.compressed_mesh = Some(CompressedMesh::load(&object.info, &mut r)?);
        }
        let offset = r.get_offset() + 24;
        r.set_offset(offset)?;
        if version[0] < 3 || (version[0] == 3 && version[1] <= 4) {
            let color_size = r.read_i32()? as usize;
            ret.colors = Vec::with_capacity(color_size * 4);
            for _ in 0..(color_size * 4) {
                ret.colors.push(r.read_u8()? as f32 / 255.0)
            }
            let collision_triangles_size = r.read_i32()? as usize;
            let offset = r.get_offset() + collision_triangles_size * 4;
            r.set_offset(offset)?;
            let _collision_vertex_count = r.read_i32()?;
        }
        let _mesh_usage_flags = r.read_i32()?;
        if version[0] > 2022 || (version[0] == 2022 && version[1] >= 1) {
            let _cooking_options = r.read_i32()?;
        }
        if version[0] >= 5 {
            let size = r.read_i32()? as usize;
            let _baked_convex_collision_mesh = r.read_u8_list(size)?;
            r.align(4)?;
            let size = r.read_i32()? as usize;
            let _baked_triangle_collision_mesh = r.read_u8_list(size)?;
            r.align(4)?;
        }
        if version[0] > 2018 || (version[0] == 2018 && version[1] >= 2) {
            let _mesh_metrics = r.read_f32_array::<2>()?;
        }
        if version[0] > 2018 || (version[0] == 2018 && version[1] >= 3) {
            r.align(4)?;
            ret.stream_data = Some(StreamingInfo::load(&object.info, &mut r)?);
        }
        ret.process_data(object)?;
        Ok(ret)
    }
}

#[derive(Debug)]
pub struct BlendShapeData {
    pub vertices: Vec<BlendShapeVertex>,
    pub shapes: Vec<MeshBlendShape>,
    pub channels: Vec<MeshBlendShapeChannel>,
    pub full_weights: Vec<f32>,
}

impl BlendShapeData {
    pub(super) fn load(object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        if object.version[0] > 4 || (object.version[0] == 4 && object.version[1] >= 3) {
            let num_verts = r.read_i32()?;
            let mut vertices = Vec::with_capacity(num_verts as usize);
            for _ in 0..num_verts {
                vertices.push(BlendShapeVertex::load(object, r)?);
            }
            let num_shapes = r.read_i32()?;
            let mut shapes = Vec::with_capacity(num_shapes as usize);
            for _ in 0..num_shapes {
                shapes.push(MeshBlendShape::load(object, r)?);
            }
            let num_channels = r.read_i32()?;
            let mut channels = Vec::with_capacity(num_channels as usize);
            for _ in 0..num_channels {
                channels.push(MeshBlendShapeChannel::load(object, r)?);
            }
            let length = r.read_i32()? as usize;
            let full_weights = r.read_f32_list(length)?;
            Ok(Self { vertices, shapes, channels, full_weights })
        } else {
            let num_shapes = r.read_i32()?;
            let mut shapes = Vec::with_capacity(num_shapes as usize);
            for _ in 0..num_shapes {
                shapes.push(MeshBlendShape::load(object, r)?);
            }
            r.align(4)?;
            let num_verts = r.read_i32()?;
            let mut vertices = Vec::with_capacity(num_verts as usize);
            for _ in 0..num_verts {
                vertices.push(BlendShapeVertex::load(object, r)?);
            }
            Ok(Self {
                vertices,
                shapes,
                channels: Vec::new(),
                full_weights: Vec::new(),
            })
        }
    }
}

#[derive(Debug)]
pub struct BlendShapeVertex {
    pub vertex: Vector3,
    pub normal: Vector3,
    pub tangent: Vector3,
    pub index: u32,
}

impl BlendShapeVertex {
    pub(super) fn load(_object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        Ok(Self {
            vertex: r.read_vector3()?,
            normal: r.read_vector3()?,
            tangent: r.read_vector3()?,
            index: r.read_u32()?,
        })
    }
}

#[derive(Debug)]
pub struct MeshBlendShape {
    pub first_vertex: u32,
    pub vertex_count: u32,
    pub has_normals: bool,
    pub has_tangent: bool,
}

impl MeshBlendShape {
    pub(super) fn load(object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        if object.version[0] == 4 && object.version[1] < 3 {
            let _name = r.read_aligned_string()?;
        }
        let first_vertex = r.read_u32()?;
        let vertex_count = r.read_u32()?;
        if object.version[0] == 4 && object.version[1] < 3 {
            let _aabb_min_delta = r.read_vector3()?;
            let _aabb_max_delta = r.read_vector3()?;
        }
        let has_normals = r.read_bool()?;
        let has_tangent = r.read_bool()?;
        if object.version[0] > 4 || (object.version[0] == 4 && object.version[1] >= 3) {
            r.align(4)?;
        }
        Ok(Self {
            first_vertex,
            vertex_count,
            has_normals,
            has_tangent,
        })
    }
}

#[derive(Debug)]
pub struct MeshBlendShapeChannel {
    pub name: String,
    pub name_hash: u32,
    pub frame_index: i32,
    pub frame_count: i32,
}

impl MeshBlendShapeChannel {
    pub(super) fn load(_object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        Ok(Self {
            name: r.read_aligned_string()?,
            name_hash: r.read_u32()?,
            frame_index: r.read_i32()?,
            frame_count: r.read_i32()?,
        })
    }
}

#[derive(Debug)]
pub struct CompressedMesh {
    pub vertices: PackedFloatVector,
    pub uv: PackedFloatVector,
    pub bind_poses: Option<PackedFloatVector>,
    pub normals: PackedFloatVector,
    pub tangents: PackedFloatVector,
    pub weights: PackedIntVector,
    pub normal_signs: PackedIntVector,
    pub tangent_signs: PackedIntVector,
    pub float_colors: Option<PackedFloatVector>,
    pub bone_indices: PackedIntVector,
    pub triangles: PackedIntVector,
    pub colors: Option<PackedIntVector>,
    pub uv_info: u32,
}

impl CompressedMesh {
    pub(super) fn load(object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        let version = object.version;
        let vertices = PackedFloatVector::load(object, r)?;
        let uv = PackedFloatVector::load(object, r)?;
        let bind_poses = if version[0] < 5 { Some(PackedFloatVector::load(object, r)?) } else { None };
        let normals = PackedFloatVector::load(object, r)?;
        let tangents = PackedFloatVector::load(object, r)?;
        let weights = PackedIntVector::load(object, r)?;
        let normal_signs = PackedIntVector::load(object, r)?;
        let tangent_signs = PackedIntVector::load(object, r)?;
        let float_colors = if version[0] < 5 { None } else { Some(PackedFloatVector::load(object, r)?) };
        let bone_indices = PackedIntVector::load(object, r)?;
        let triangles = PackedIntVector::load(object, r)?;
        let mut colors = None;
        let mut uv_info = 0;
        if version[0] > 3 || (version[0] == 3 && version[1] >= 5) {
            if version[0] < 5 {
                colors = Some(PackedIntVector::load(object, r)?);
            } else {
                uv_info = r.read_u32()?;
            }
        }
        Ok(Self {
            vertices,
            uv,
            bind_poses,
            normals,
            tangents,
            weights,
            normal_signs,
            tangent_signs,
            float_colors,
            bone_indices,
            triangles,
            colors,
            uv_info,
        })
    }
}

#[derive(Default, Debug)]
pub struct SubMesh {
    pub first_bytes: u32,
    pub index_count: u32,
    pub topology: GfxPrimitiveType,
    pub triangle_count: u32,
    pub base_vertex: u32,
    pub first_vertex: u32,
    pub vertex_count: u32,
    pub local_aabb: Option<AnimationClip>,
}

impl SubMesh {
    pub(super) fn load(object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        let mut result = Self::default();
        let version = object.version;
        result.first_bytes = r.read_u32()?;
        result.index_count = r.read_u32()?;
        result.topology = r.read_i32()?.try_into().or(Err(UnityError::InvalidValue))?;
        if version[0] < 4 {
            result.triangle_count = r.read_u32()?;
        }

        if version[0] > 2017 || (version[0] == 2017 && version[1] >= 3) {
            result.base_vertex = r.read_u32()?;
        }

        if version[0] >= 3 {
            result.first_vertex = r.read_u32()?;
            result.vertex_count = r.read_u32()?;
            result.local_aabb = Some(AnimationClip::load(r)?);
        }
        Ok(result)
    }
}

#[derive(Default, Debug)]
pub struct ChannelInfo {
    pub stream: u8,
    pub offset: u8,
    pub format: u8,
    pub dimension: u8,
}

impl ChannelInfo {
    pub(super) fn load(_object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        Ok(Self {
            stream: r.read_u8()?,
            offset: r.read_u8()?,
            format: r.read_u8()?,
            dimension: r.read_u8()? & 0xF,
        })
    }
}

#[derive(Default, Debug)]
pub struct StreamInfo {
    pub channel_mask: u8,
    pub offset: u8,
    pub stride: u8,
    pub align: u8,
    pub divider_op: u8,
    pub frequency: u16,
}

impl StreamInfo {
    pub(super) fn load(object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        let version = object.version;
        let mut result = Self {
            channel_mask: r.read_u8()?,
            offset: r.read_u8()?,
            ..Self::default()
        };

        if version[0] < 4 {
            result.stride = r.read_u32()? as u8;
            result.align = r.read_u32()? as u8;
        } else {
            result.stride = r.read_u8()?;
            result.divider_op = r.read_u8()?;
            result.frequency = r.read_u16()?;
        }
        Ok(result)
    }
}
#[derive(Default, Debug)]
pub struct VertexData {
    pub current_channels: u8,
    pub vertex_count: usize,
    pub channels: Vec<ChannelInfo>,
    pub streams: Vec<StreamInfo>,
    pub data_size: Vec<u8>,
}

impl VertexData {
    pub(super) fn load(object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        let version = object.version;
        let mut result = Self::default();
        if version[0] < 2018 {
            result.current_channels = r.read_u32()? as u8;
        }

        result.vertex_count = r.read_u32()? as usize;

        if version[0] >= 4 {
            let size = r.read_i32()?;
            for _ in 0..size {
                result.channels.push(ChannelInfo::load(object, r)?)
            }
        }
        if version[0] < 5 {
            if version[0] < 4 {
                result.streams = Vec::with_capacity(4);
            } else {
                result.streams = Vec::with_capacity(r.read_i32()? as usize);
            }
            for _ in 0..result.streams.capacity() {
                result.streams.push(StreamInfo::load(object, r)?)
            }
            if version[0] < 4 {
                result.get_channels(version)?;
            }
        } else {
            result.get_streams(version)?;
        }
        let size = r.read_i32()?;
        result.data_size = r.read_u8_list(size as usize)?;
        Ok(result)
    }

    fn get_channels(&mut self, _version: [i32; 4]) -> UnityResult<()> {
        self.channels = Vec::with_capacity(6);
        for _ in 0..6 {
            self.channels.push(ChannelInfo::default())
        }
        for s in 0..self.streams.len() {
            let channel_mask = self.streams[s].channel_mask;
            let offset = 0;
            for i in 0..6 {
                if (channel_mask >> i) & 0x1 == 0 {
                    continue;
                }
                let channel = &mut self.channels[i];
                channel.stream = s as u8;
                channel.offset = offset;
                match i {
                    0 | 1 => {
                        channel.format = 0;
                        channel.dimension = 3;
                        break;
                    }
                    2 => {
                        channel.format = 2;
                        channel.dimension = 4;
                        break;
                    }
                    3 | 4 => {
                        channel.format = 0;
                        channel.dimension = 2;
                        break;
                    }
                    5 => {
                        channel.format = 0;
                        channel.dimension = 4;
                        break;
                    }
                    _ => unreachable!(),
                }
                //offset += (m_Channel.dimension
                //    * VertexFormat::load(m_Channel.format, _version)?.get_format_size())
            }
        }
        Ok(())
    }

    fn get_streams(&mut self, version: [i32; 4]) -> UnityResult<()> {
        let stream_count = {
            let mut max = 0;
            for i in &self.channels {
                if i.stream > max {
                    max = i.stream
                }
            }
            max + 1
        };
        self.streams = Vec::with_capacity(stream_count as usize);
        let mut offset = std::num::Wrapping(0);
        for s in 0..stream_count {
            let mut chn_mask = 0;
            let mut stride = 0;
            for chn in 0..self.channels.len() {
                let channel = &self.channels[chn];
                if channel.stream == s && channel.dimension > 0 {
                    chn_mask |= 1u8 << chn;
                    stride += channel.dimension * VertexFormat::load(channel.format, version)?.get_format_size()
                }
            }
            self.streams.push(StreamInfo {
                channel_mask: chn_mask,
                offset: offset.0,
                stride,
                align: 0,
                frequency: 0,
                divider_op: 0,
            });
            offset += std::num::Wrapping(self.vertex_count as u8) * std::num::Wrapping(stride);
            offset += std::num::Wrapping((16u8 - 1u8) & (!(16u8 - 1u8)));
        }
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum VertexFormat {
    Float,
    Float16,
    UNorm8,
    SNorm8,
    UNorm16,
    SNorm16,
    UInt8,
    SInt8,
    UInt16,
    SInt16,
    UInt32,
    SInt32,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum VertexChannelFormat {
    Float,
    Float16,
    Color,
    Byte,
    UInt32,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum VertexFormat2017 {
    Float,
    Float16,
    Color,
    UNorm8,
    SNorm8,
    UNorm16,
    SNorm16,
    UInt8,
    SInt8,
    UInt16,
    SInt16,
    UInt32,
    SInt32,
}
impl VertexFormat {
    fn load(format: u8, version: [i32; 4]) -> UnityResult<Self> {
        if version[0] < 2017 {
            let result = match VertexChannelFormat::try_from(format).or(Err(UnityError::InvalidValue))? {
                VertexChannelFormat::Float => VertexFormat::Float,
                VertexChannelFormat::Float16 => VertexFormat::Float16,
                VertexChannelFormat::Color => VertexFormat::UNorm8,
                VertexChannelFormat::Byte => VertexFormat::UInt8,
                VertexChannelFormat::UInt32 => VertexFormat::UInt32,
            };
            return Ok(result);
        }
        if version[0] < 2019 {
            let result = match VertexFormat2017::try_from(format).or(Err(UnityError::InvalidValue))? {
                VertexFormat2017::Float => VertexFormat::Float,
                VertexFormat2017::Float16 => VertexFormat::Float16,
                VertexFormat2017::Color => VertexFormat::UNorm8,
                VertexFormat2017::UNorm8 => VertexFormat::UNorm8,
                VertexFormat2017::SNorm8 => VertexFormat::SNorm8,
                VertexFormat2017::UNorm16 => VertexFormat::UNorm16,
                VertexFormat2017::SNorm16 => VertexFormat::SNorm16,
                VertexFormat2017::UInt8 => VertexFormat::UInt8,
                VertexFormat2017::SInt8 => VertexFormat::SInt8,
                VertexFormat2017::UInt16 => VertexFormat::UInt16,
                VertexFormat2017::SInt16 => VertexFormat::SInt16,
                VertexFormat2017::UInt32 => VertexFormat::UInt32,
                VertexFormat2017::SInt32 => VertexFormat::SInt32,
            };
            return Ok(result);
        }
        VertexFormat::try_from(format).or(Err(UnityError::InvalidValue))
    }

    fn get_format_size(&self) -> u8 {
        match *self {
            VertexFormat::Float | VertexFormat::UInt32 | VertexFormat::SInt32 => 4,

            VertexFormat::Float16 | VertexFormat::UNorm16 | VertexFormat::SNorm16 | VertexFormat::UInt16 | VertexFormat::SInt16 => 2,

            VertexFormat::UNorm8 | VertexFormat::SNorm8 | VertexFormat::UInt8 | VertexFormat::SInt8 => 1,
        }
    }

    fn is_int(&self) -> bool {
        match self {
            VertexFormat::Float => false,
            VertexFormat::Float16 => false,
            VertexFormat::UNorm8 => false,
            VertexFormat::SNorm8 => false,
            VertexFormat::UNorm16 => false,
            VertexFormat::SNorm16 => false,
            VertexFormat::UInt8 => true,
            VertexFormat::SInt8 => true,
            VertexFormat::UInt16 => true,
            VertexFormat::SInt16 => true,
            VertexFormat::UInt32 => true,
            VertexFormat::SInt32 => true,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct BoneWeights4 {
    pub weight: [f32; 4],
    pub bone_index: [i32; 4],
}

impl BoneWeights4 {
    pub(super) fn load(_object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        let weight = r.read_f32_array::<4>()?;
        let bone_index = r.read_i32_array::<4>()?;
        Ok(Self { weight, bone_index })
    }
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct MinMaxAABB {
    pub min: Vector3,
    pub max: Vector3,
}

impl MinMaxAABB {
    pub(super) fn load(_object: &ObjectInfo, r: &mut Reader) -> UnityResult<Self> {
        let min = r.read_vector3()?;
        let max = r.read_vector3()?;
        Ok(Self { min, max })
    }
}

fn bytes_to_f32_vec(data: &[u8], format: VertexFormat) -> Vec<f32> {
    match format {
        VertexFormat::Float => {
            let mut buf = [0u8; 4];
            data.chunks_exact(4)
                .map(|x| {
                    buf.copy_from_slice(x);
                    f32::from_le_bytes(buf)
                })
                .collect()
        }
        VertexFormat::Float16 => {
            let mut buf = [0u8; 2];
            data.chunks_exact(2)
                .map(|x| {
                    buf.copy_from_slice(x);
                    half::f16::from_le_bytes(buf).to_f32()
                })
                .collect()
        }
        VertexFormat::UNorm8 => data.iter().copied().map(|x| x as f32 / 255.0).collect(),
        VertexFormat::SNorm8 => data.iter().copied().map(|x| (x as i8 as f32 / 127.0).max(-1.0)).collect(),
        VertexFormat::UNorm16 => {
            let mut buf = [0u8; 2];
            data.chunks_exact(2)
                .map(|x| {
                    buf.copy_from_slice(x);
                    u16::from_le_bytes(buf) as f32 / 65535.0
                })
                .collect()
        }
        VertexFormat::SNorm16 => {
            let mut buf = [0u8; 2];
            data.chunks_exact(2)
                .map(|x| {
                    buf.copy_from_slice(x);
                    (i16::from_le_bytes(buf) as f32 / 32767.0).max(-1.0)
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn bytes_to_i32_vec(data: &[u8], format: VertexFormat) -> Vec<i32> {
    match format {
        VertexFormat::UInt8 | VertexFormat::SInt8 => data.iter().copied().map(|x| x as i32).collect(),
        VertexFormat::UInt16 | VertexFormat::SInt16 => {
            let mut buf = [0u8; 2];
            data.chunks_exact(2)
                .map(|x| {
                    buf.copy_from_slice(x);
                    i16::from_le_bytes(buf) as i32
                })
                .collect()
        }
        VertexFormat::UInt32 | VertexFormat::SInt32 => {
            let mut buf = [0u8; 4];
            data.chunks_exact(4)
                .map(|x| {
                    buf.copy_from_slice(x);
                    i32::from_le_bytes(buf)
                })
                .collect()
        }
        _ => Vec::new(),
    }
}
