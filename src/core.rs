use crate::util::compare_version;
use crate::util::{compression_file, decompress_file};
use anyhow::{anyhow, Result};
use memchr::memmem;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// 缓冲区大小（512KB）
pub const BUFFER_SIZE: usize = 1024 * 512;

/// 资源最大大小（1024GB）
pub const MAX_LENGTH_SIZE: u64 = 1024 * 1024 * 1024 * 1024;

/// 最大id长度
pub const MAX_ID_LENGTH: usize = 64;

/// 最大文件名长度
pub const MAX_NAME_LENGTH: usize = 255;

/// 压缩模式
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum CompressMode {
    /// 无压缩
    None,
    /// 有压缩
    Compress,
}

/// 资源文件魔数
const RESOURCE_MAGIC: &[u8] = &[
    0x89, b'O', b'v', b'e', b'r', b'l', b'a', b'y', b'D', b'a', b't', b'a', 0x0d, 0x0a, 0x1a, 0x0a,
];

/// 资源文件头
#[derive(Serialize, Deserialize, Debug)]
pub struct ResourceHead {
    /// 资源版本(不应与程序版本号绑定)
    version: String,
    /// 资源ID
    id: String,
    /// 资源文件名
    name: String,
    /// 资源长度
    length: String,
    /// 资源大小
    size: String,
    /// 压缩模式
    compress: CompressMode,
}

impl ResourceHead {
    pub(crate) fn default() -> Self {
        ResourceHead::new("", 0, 0, "", CompressMode::None)
    }

    /// 获取文件头魔数（标识）
    pub fn get_head(&self) -> &'static [u8] {
        RESOURCE_MAGIC
    }

    /// 创建资源文件头
    ///
    /// # 参数
    /// - `id`: 资源ID
    /// - `length`: 资源长度
    /// - `size`: 资源大小
    /// - `name`: 资源文件名
    /// - `compress`: 压缩模式
    ///
    /// # 返回值
    /// - ResourceHead: 资源文件头
    pub fn new(id: &str, length: u64, size: u64, name: &str, compress: CompressMode) -> Self {
        // 验证输入字符数不超过限制
        assert!(
            id.chars().count() <= MAX_ID_LENGTH,
            "Resource ID exceeds maximum length of {} characters",
            MAX_ID_LENGTH
        );
        assert!(
            name.chars().count() <= MAX_NAME_LENGTH,
            "Resource name exceeds maximum length of {} characters",
            MAX_NAME_LENGTH
        );

        ResourceHead {
            version: "1.0.0".to_string(),
            id: id.to_string(),
            name: name.to_string(),
            length: format!(
                "{:0>width$}",
                length,
                width = MAX_LENGTH_SIZE.to_string().len()
            ),
            size: format!(
                "{:0>width$}",
                size,
                width = MAX_LENGTH_SIZE.to_string().len()
            ),
            compress,
        }
    }

    /// 获取文件头长度（序列化后的字节数）
    pub fn get_len(&self) -> usize {
        self.to_bytes()
            .expect("Failed to serialize resource header")
            .len()
    }

    /// 转换为字节
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(&self)?)
    }

    /// 将字节解析为当前数据
    pub fn from(data: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(data)?)
    }

    /// 获取资源ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 获取资源名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取资源大小
    pub fn size(&self) -> &str {
        &self.size
    }

    /// 获取压缩模式
    pub fn compress(&self) -> CompressMode {
        self.compress
    }
}

/// 资源文件尾(ODEND)
const END_IDENTIFIER: [u8; 5] = [0x4F, 0x44, 0x45, 0x4E, 0x44];

/// 增加资源(Overlay 附加数据)
///
/// # 参数
/// - `target_file_path`: 目标文件路径
/// - `source_file_path`: 资源文件路径
/// - `id`: 资源ID（不可重复）
/// - `compression_grade`: 压缩等级(0-9)
///     - 0: 不压缩
///     - 1: 为优化编码的最佳速度
///     - 9: 针对正在编码的数据大小进行优化。
/// - `output_path`: 输出文件路径(可选)
///
/// # 返回值
/// - Ok(())
/// - Err(err)
pub fn add_resource(
    target_file_path: &Path,
    source_file_path: &Path,
    id: &str,
    compression_grade: Option<u32>,
    output_path: Option<&Path>,
) -> Result<()> {
    // 打开资源文件
    let source_file_path_buf = if source_file_path.is_relative() {
        target_file_path
            .parent()
            .ok_or_else(|| anyhow!("Target file has no parent directory"))?
            .join(source_file_path)
    } else {
        source_file_path.to_path_buf()
    };
    let mut source_file = File::open(&source_file_path_buf)?;
    let source_name = &source_file_path_buf
        .file_name()
        .ok_or_else(|| anyhow!("Source file has no valid filename"))?
        .to_string_lossy();

    // 验证资源文件大小
    let _source_size = source_file.metadata()?.len();

    // 处理压缩资源
    let temp_file_path = &*source_file_path_buf
        .parent()
        .ok_or_else(|| anyhow!("Source file has no parent directory"))?
        .join("temp");
    if let Some(grage) = compression_grade {
        compression_file(&source_file_path_buf, temp_file_path, grage)?;
        source_file = File::open(temp_file_path)?;
    }
    let source_length = source_file.metadata()?.len();

    //以追加模式打开目标文件
    let target_file_path_buf = if let Some(output_path_param) = output_path {
        // 处理相对路径
        let output_path_buf = if output_path_param.is_relative() {
            target_file_path
                .parent()
                .ok_or_else(|| anyhow!("Target file has no parent directory"))?
                .join(output_path_param)
        } else {
            output_path_param.to_path_buf()
        };
        fs::copy(target_file_path, &output_path_buf)?;
        output_path_buf
    } else {
        target_file_path.to_path_buf()
    };

    let mut target_file = OpenOptions::new()
        .append(true)
        .open(&target_file_path_buf)?;

    let compress_mode = match compression_grade.is_some() {
        true => CompressMode::Compress,
        false => CompressMode::None,
    };

    // 插入魔数标识
    target_file.write_all(RESOURCE_MAGIC)?;

    // 插入资源头
    let head = ResourceHead::new(id, source_length, source_length, source_name, compress_mode)
        .to_bytes()?;
    target_file.write_all(&head)?;

    // 缓冲区
    let mut buffer = [0u8; BUFFER_SIZE];

    // 循环读取并写入资源文件
    loop {
        let nbytes = source_file.read(&mut buffer)?;
        target_file.write_all(&buffer[..nbytes])?;
        if nbytes < buffer.len() {
            break;
        }
    }

    // 插入尾部标识
    target_file.write_all(&END_IDENTIFIER)?;
    // 确保所有数据都写入磁盘
    target_file.flush()?;

    // 清除临时压缩资源
    if temp_file_path.exists() {
        fs::remove_file(temp_file_path)?;
    }
    Ok(())
}

/// 释放资源
///
/// # 参数
/// - `target_file_path`: 目标文件路径
/// - `id`: 资源ID
/// - `output_path`: 输出路径
///
/// # 返回值
/// - Ok(())
/// - Err(err)
pub fn export_resource(target_file_path: &Path, id: &str, output_path: &Path) -> Result<()> {
    let magic_finder = memmem::Finder::new(RESOURCE_MAGIC);

    // 打开目标文件
    let mut source_file = File::open(target_file_path)?;
    let file_len = source_file.metadata()?.len();

    // 优化：使用更大的缓冲区，并保留重叠区域以避免遗漏跨边界的魔数
    const SEARCH_BUFFER_SIZE: usize = 1024 * 512; // 512KB 搜索缓冲区
    const MAX_HEADER_SIZE: usize = 4096; // 最大可能的资源头大小
    let overlap_size = MAX_HEADER_SIZE + RESOURCE_MAGIC.len(); // 重叠区域大小

    let mut buffer = Vec::with_capacity(SEARCH_BUFFER_SIZE + overlap_size);
    let mut file_offset: u64 = 0; // 当前缓冲区在文件中的起始位置

    loop {
        // 读取数据到缓冲区
        buffer.clear();
        buffer.resize(SEARCH_BUFFER_SIZE, 0);

        source_file.seek(SeekFrom::Start(file_offset))?;
        let bytes_read = source_file.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        if bytes_read == 0 {
            return Err(anyhow!("Resource not found"));
        }

        // 在缓冲区中搜索魔数
        let mut search_start = 0;
        while let Some(relative_pos) = magic_finder.find(&buffer[search_start..]) {
            let absolute_pos = search_start + relative_pos;
            let resource_start = file_offset as usize + absolute_pos;

            // 尝试读取资源头
            // 需要跳过魔数本身，然后才解析 ResourceHead
            let magic_len = RESOURCE_MAGIC.len();
            let config = if absolute_pos + magic_len + MAX_HEADER_SIZE <= buffer.len() {
                // 资源头完全在当前缓冲区内，跳过魔数后解析
                ResourceHead::from(&buffer[absolute_pos + magic_len..])
            } else {
                // 资源头可能超出缓冲区，需要从文件读取
                source_file.seek(SeekFrom::Start((resource_start + magic_len) as u64))?;
                let mut header_buffer = vec![0u8; MAX_HEADER_SIZE];
                // 读取尽可能多的字节，而不是要求完整的 MAX_HEADER_SIZE
                let n = source_file.read(&mut header_buffer)?;
                if n == 0 {
                    search_start = absolute_pos + 1;
                    continue;
                }
                ResourceHead::from(&header_buffer[..n])
            };

            let config = match config {
                Ok(c) => c,
                Err(_) => {
                    // 不是有效的资源头，继续搜索
                    search_start = absolute_pos + 1;
                    continue;
                }
            };

            // 检查 ID 是否匹配
            if config.id.trim() == id.trim() {
                // 找到目标资源

                // 验证版本
                let default_resource_head = ResourceHead::default();
                let version_ordering =
                    compare_version(&config.version, &default_resource_head.version)?;
                if version_ordering.is_ne() {
                    return Err(anyhow!(
                        "Resource version mismatch: file has {}, program supports {}",
                        &config.version,
                        &default_resource_head.version
                    ));
                }

                let magic_len = RESOURCE_MAGIC.len();
                let header_len = config.get_len();
                let resource_length = config
                    .length
                    .trim()
                    .parse::<usize>()
                    .map_err(|e| anyhow!("Failed to parse resource length: {}", e))?;

                // 验证资源完整性（检查结束标识）
                let end_pos = resource_start + magic_len + header_len + resource_length;
                if end_pos + END_IDENTIFIER.len() > file_len as usize {
                    return Err(anyhow!("Resource extends beyond file boundary"));
                }

                source_file.seek(SeekFrom::Start((end_pos) as u64))?;
                let mut end_buffer = [0u8; END_IDENTIFIER.len()];
                source_file.read_exact(&mut end_buffer)?;
                if end_buffer != END_IDENTIFIER {
                    return Err(anyhow!(
                        "Resource end marker not found - file may be corrupted"
                    ));
                }

                // 准备输出路径
                let output_path_buf = if output_path.is_relative() {
                    target_file_path
                        .parent()
                        .ok_or_else(|| anyhow!("Target file has no parent directory"))?
                        .join(output_path)
                } else {
                    output_path.to_path_buf()
                };
                let output_path_buf = if output_path_buf.is_dir() {
                    output_path_buf.join(config.name.trim())
                } else {
                    output_path_buf
                };

                // 读取资源数据
                source_file.seek(SeekFrom::Start(
                    (resource_start + magic_len + header_len) as u64,
                ))?;
                let mut output_file = File::create(&output_path_buf)?;

                // 使用固定大小的缓冲区读取资源数据
                let mut data_buffer = vec![0u8; BUFFER_SIZE.min(resource_length)];
                let mut remaining = resource_length;

                while remaining > 0 {
                    let to_read = data_buffer.len().min(remaining);
                    data_buffer.truncate(to_read);
                    source_file.read_exact(&mut data_buffer)?;
                    output_file.write_all(&data_buffer)?;
                    remaining -= to_read;
                }

                // 处理压缩资源
                if config.compress == CompressMode::Compress {
                    let actual_file = output_path_buf
                        .parent()
                        .ok_or_else(|| anyhow!("Output path has no parent directory"))?
                        .join("actualFile");
                    decompress_file(&output_path_buf, &actual_file)?;
                    fs::remove_file(&output_path_buf)?;
                    fs::rename(actual_file, &output_path_buf)?;
                }

                // 验证输出文件大小
                let expected_size = config.size.trim().parse::<u64>()?;
                if output_file.metadata()?.len() != expected_size {
                    fs::remove_file(&output_path_buf)?;
                    return Err(anyhow!(
                        "Exported file size mismatch: expected {}, got {}",
                        expected_size,
                        output_file.metadata()?.len()
                    ));
                }

                return Ok(());
            }

            // ID 不匹配，继续搜索下一个可能的魔数
            search_start = absolute_pos + 1;
        }

        // 移动到下一个位置，保留重叠区域以防魔数跨边界
        if file_offset as usize + SEARCH_BUFFER_SIZE >= file_len as usize {
            // 已到文件末尾
            return Err(anyhow!("Resource not found"));
        }

        file_offset += SEARCH_BUFFER_SIZE as u64 - overlap_size as u64;
    }
}

/// 寻找资源配置 - 从头至尾
///
/// # 参数
/// - `target_file_path`: 目标文件路径
/// - `callback`: 回调函数(配置位置, 资源配置)
///
/// # 返回值
/// - `Vec<ResourceHead>`: 资源配置列表
/// - Err(err)
pub fn find_resources_config(
    target_file_path: &Path,
    callback: fn(start_size: usize, config: &ResourceHead),
) -> Result<Vec<ResourceHead>> {
    let magic_finder = memmem::Finder::new(RESOURCE_MAGIC);

    // 打开目标文件
    let mut source_file = File::open(target_file_path)?;
    let file_len = source_file.metadata()?.len();

    // 优化：使用更大的缓冲区，并保留重叠区域
    const SEARCH_BUFFER_SIZE: usize = 1024 * 512; // 512KB 搜索缓冲区
    const MAX_HEADER_SIZE: usize = 4096; // 最大可能的资源头大小
    let overlap_size = MAX_HEADER_SIZE + RESOURCE_MAGIC.len(); // 重叠区域大小

    let mut buffer = Vec::with_capacity(SEARCH_BUFFER_SIZE + overlap_size);
    let mut file_offset: u64 = 0;

    let mut configs = Vec::new();

    loop {
        // 读取数据到缓冲区
        buffer.clear();
        buffer.resize(SEARCH_BUFFER_SIZE, 0);

        source_file.seek(SeekFrom::Start(file_offset))?;
        let bytes_read = source_file.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        if bytes_read == 0 {
            break;
        }

        // 在缓冲区中搜索魔数
        let mut search_start = 0;
        while let Some(relative_pos) = magic_finder.find(&buffer[search_start..]) {
            let absolute_pos = search_start + relative_pos;
            let resource_start = file_offset as usize + absolute_pos;

            // 尝试读取资源头
            // 需要跳过魔数本身，然后才解析 ResourceHead
            let magic_len = RESOURCE_MAGIC.len();
            let config = if absolute_pos + magic_len + MAX_HEADER_SIZE <= buffer.len() {
                // 资源头完全在当前缓冲区内，跳过魔数后解析
                ResourceHead::from(&buffer[absolute_pos + magic_len..])
            } else {
                // 资源头可能超出缓冲区，从文件读取
                source_file.seek(SeekFrom::Start((resource_start + magic_len) as u64))?;
                let mut header_buffer = vec![0u8; MAX_HEADER_SIZE];
                // 读取尽可能多的字节，而不是要求完整的 MAX_HEADER_SIZE
                let n = source_file.read(&mut header_buffer)?;
                if n == 0 {
                    search_start = absolute_pos + 1;
                    continue;
                }
                ResourceHead::from(&header_buffer[..n])
            };

            let config = match config {
                Ok(c) => c,
                Err(_) => {
                    // 不是有效的资源头，继续搜索
                    search_start = absolute_pos + 1;
                    continue;
                }
            };

            callback(resource_start, &config);
            configs.push(config);

            // 继续搜索下一个可能的魔数
            search_start = absolute_pos + 1;
        }

        // 移动到下一个位置，保留重叠区域
        if file_offset as usize + SEARCH_BUFFER_SIZE >= file_len as usize {
            break;
        }

        file_offset += SEARCH_BUFFER_SIZE as u64 - overlap_size as u64;
    }

    Ok(configs)
}

/// 寻找字节（速度较慢）
///
/// # 参数
/// - `haystack`: 主字符串
/// - `needle`: 子字符串
///
/// # 返回值
/// - `Option<usize>`: 返回找到的字节位置
fn _find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// 删除资源
///
/// # 参数
/// - `target_file_path`: 目标文件路径
/// - `id`: 资源ID
/// - `output_path`: 输出文件路径(可选)
///
/// # 返回值
/// - Ok(())
/// - Err(err)
pub fn remove_resource(
    target_file_path: &Path,
    id: &str,
    output_path: Option<&Path>,
) -> Result<()> {
    let magic_finder = memmem::Finder::new(RESOURCE_MAGIC);

    // 打开目标文件
    let mut source_file = File::open(target_file_path)?;
    let file_len = source_file.metadata()?.len();

    // 读取整个文件到内存
    let mut file_data = Vec::with_capacity(file_len as usize);
    source_file.read_to_end(&mut file_data)?;

    // 搜索目标资源
    const MAX_HEADER_SIZE: usize = 4096;
    let mut resource_start: Option<usize> = None;
    let mut resource_end: Option<usize> = None;

    let mut search_pos = 0;
    while let Some(relative_pos) = magic_finder.find(&file_data[search_pos..]) {
        let absolute_pos = search_pos + relative_pos;

        // 尝试读取资源头
        let magic_len = RESOURCE_MAGIC.len();
        let config = if absolute_pos + magic_len + MAX_HEADER_SIZE <= file_data.len() {
            ResourceHead::from(&file_data[absolute_pos + magic_len..])
        } else {
            let available = file_data.len() - absolute_pos - magic_len;
            if available == 0 {
                search_pos = absolute_pos + 1;
                continue;
            }
            ResourceHead::from(&file_data[absolute_pos + magic_len..])
        };

        let config = match config {
            Ok(c) => c,
            Err(_) => {
                search_pos = absolute_pos + 1;
                continue;
            }
        };

        // 检查 ID 是否匹配
        if config.id.trim() == id.trim() {
            let magic_len = RESOURCE_MAGIC.len();
            let header_len = config.get_len();
            let resource_length = config
                .length
                .trim()
                .parse::<usize>()
                .map_err(|e| anyhow!("Failed to parse resource length: {}", e))?;

            // 计算资源结束位置
            let end_pos = absolute_pos + magic_len + header_len + resource_length;

            // 验证结束标识
            if end_pos + END_IDENTIFIER.len() > file_data.len() {
                return Err(anyhow!("Resource extends beyond file boundary"));
            }

            if file_data[end_pos..end_pos + END_IDENTIFIER.len()] != END_IDENTIFIER {
                return Err(anyhow!(
                    "Resource end marker not found - file may be corrupted"
                ));
            }

            resource_start = Some(absolute_pos);
            resource_end = Some(end_pos + END_IDENTIFIER.len());
            break;
        }

        search_pos = absolute_pos + 1;
    }

    let (start, end) = match (resource_start, resource_end) {
        (Some(s), Some(e)) => (s, e),
        _ => return Err(anyhow!("Resource not found")),
    };

    // 构建新文件数据（移除资源部分）
    let mut new_data = Vec::with_capacity(file_data.len() - (end - start));
    new_data.extend_from_slice(&file_data[..start]);
    new_data.extend_from_slice(&file_data[end..]);

    // 确定输出路径
    let output_path_buf = if let Some(output_path_param) = output_path {
        if output_path_param.is_relative() {
            target_file_path
                .parent()
                .ok_or_else(|| anyhow!("Target file has no parent directory"))?
                .join(output_path_param)
        } else {
            output_path_param.to_path_buf()
        }
    } else {
        target_file_path.to_path_buf()
    };

    // 写入新文件
    let mut output_file = File::create(&output_path_buf)?;
    output_file.write_all(&new_data)?;
    output_file.flush()?;

    Ok(())
}
