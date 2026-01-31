use anyhow::{anyhow, Result};
use flate2::write::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::cmp::Ordering;
use std::fs::File;
use std::io::copy;
use std::io::BufReader;
use std::path::Path;

/// 压缩文件
///
/// # 参数
/// - `file_path`: 压缩文件路径
/// - `output_path`: 输出路径
/// - `compression_grade`: 压缩等级(0-9)
///     - 0: 不压缩
///     - 1: 为优化编码的最佳速度
///     - 9: 针对正在编码的数据大小进行优化。
///
/// # 返回值
/// - `Ok(())`: 成功
/// - `Err(anyhow!("Error message"))`: 失败
pub fn compression_file(
    file_path: &Path,
    output_path: &Path,
    compression_grade: u32,
) -> Result<()> {
    let mut input = BufReader::new(File::open(file_path)?);
    let output = File::create(output_path)?;
    let mut encoder = GzEncoder::new(output, Compression::new(compression_grade));
    copy(&mut input, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

/// 还原压缩文件
///
/// # 参数
/// - `file_path`: 压缩文件路径
/// - `output_path`: 输出路径
///
/// # 返回值
/// - `Ok(())`: 成功
/// - `Err(anyhow!("Error message"))`: 失败
pub fn decompress_file(file_path: &Path, output_path: &Path) -> Result<()> {
    let mut input = BufReader::new(File::open(file_path)?);
    let output = File::create(output_path)?;
    let mut decoder = GzDecoder::new(output);
    copy(&mut input, &mut decoder)?;
    decoder.finish()?;
    Ok(())
}

/// 比较版本号大小
///
/// # 参数
/// - `version1`: 版本1
/// - `version2`: 版本2
///
/// # 返回值
/// - `Ordering::Greater`: 版本1 大于版本2
/// - `Ordering::Less`: 版本1 小于版本2
/// - `Ordering::Equal`: 版本1 等于版本2
pub fn compare_version(version1: &str, version2: &str) -> Result<Ordering> {
    let nums1: Vec<&str> = version1.split('.').collect();
    let nums2: Vec<&str> = version2.split('.').collect();
    let n1 = nums1.len();
    let n2 = nums2.len();

    // 比较版本
    for i in 0..std::cmp::max(n1, n2) {
        let i1 = if i < n1 {
            nums1[i]
                .parse::<i32>()
                .map_err(|_| anyhow!("Invalid version number: {}", nums1[i]))?
        } else {
            0
        };
        let i2 = if i < n2 {
            nums2[i]
                .parse::<i32>()
                .map_err(|_| anyhow!("Invalid version number: {}", nums2[i]))?
        } else {
            0
        };
        if i1 != i2 {
            return Ok(if i1 > i2 {
                Ordering::Greater
            } else {
                Ordering::Less
            });
        }
    }
    // 版本相等
    Ok(Ordering::Equal)
}
