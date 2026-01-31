use crate::core::{
    add_resource, export_resource, find_resources_config, remove_resource, CompressMode,
    ResourceHead,
};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// 测试 ResourceHead 序列化/反序列化
#[test]
fn test_resourcehead_serialization() {
    let head = ResourceHead::new("test001", 27, 27, "resource.bin", CompressMode::None);
    let serialized = head.to_bytes().unwrap();
    let deserialized = ResourceHead::from(&serialized).unwrap();

    assert_eq!(head.id(), deserialized.id());
    assert_eq!(head.name(), deserialized.name());
    assert_eq!(head.compress(), deserialized.compress());
}

/// 诊断测试：创建一个简单的文件，添加资源、查找资源、导出资源、删除资源
#[test]
fn diagnostic_test() {
    // 创建测试目录
    let test_dir = PathBuf::from(r"D:\Project\back-end\Rust\Appender\test_diag");
    fs::create_dir_all(&test_dir).unwrap();

    // 创建一个简单的目标文件
    let target_file = test_dir.join("target.bin");
    let mut f = fs::File::create(&target_file).unwrap();
    f.write_all(b"Hello, this is a test file!").unwrap();
    f.flush().unwrap();
    drop(f);

    // 验证原始文件大小
    let original_size = fs::metadata(&target_file).unwrap().len();
    assert_eq!(original_size, 27);

    // 创建资源文件
    let source_file = test_dir.join("resource.bin");
    let resource_data = b"This is the resource data!";
    let mut f = fs::File::create(&source_file).unwrap();
    f.write_all(resource_data).unwrap();
    f.flush().unwrap();
    drop(f);

    // 验证资源文件大小
    let source_size = fs::metadata(&source_file).unwrap().len();
    assert_eq!(source_size, resource_data.len() as u64);

    println!("=== 步骤 1: 创建文件 ===");
    println!("  目标文件: {:?}", target_file.file_name());
    println!("  资源文件: {:?}", source_file.file_name());
    println!("  目标文件大小: {} 字节", original_size);
    println!("  资源数据大小: {} 字节", resource_data.len());

    // 步骤 2: 添加资源
    println!("\n=== 步骤 2: 添加资源 ===");
    let resource_id = "test001";
    add_resource(&target_file, &source_file, resource_id, None, None).unwrap();

    let size_after_add = fs::metadata(&target_file).unwrap().len();
    println!("  ✓ 添加成功 (ID: {})", resource_id);
    println!("  文件大小: {} -> {} 字节 (+{})", original_size, size_after_add, size_after_add - original_size);

    // 步骤 3: 查找资源
    println!("\n=== 步骤 3: 查找资源 ===");
    let configs = find_resources_config(&target_file, |_pos, config| {
        println!("  - ID: {}, 名称: {}, 大小: {} 字节",
            config.id().trim(),
            config.name().trim(),
            config.size().trim()
        );
    })
    .unwrap();
    println!("  ✓ 共找到 {} 个资源", configs.len());
    assert!(!configs.is_empty(), "应该找到至少一个资源");

    // 步骤 4: 导出资源
    println!("\n=== 步骤 4: 导出资源 ===");
    let output_file = test_dir.join("exported.bin");
    export_resource(&target_file, resource_id, &output_file).unwrap();

    let exported_data = fs::read(&output_file).unwrap();
    let original_data = fs::read(&source_file).unwrap();
    assert_eq!(exported_data, original_data);
    println!("  ✓ 导出成功，内容验证通过");

    // 步骤 5: 删除资源
    println!("\n=== 步骤 5: 删除资源 ===");
    let size_before_remove = fs::metadata(&target_file).unwrap().len();
    remove_resource(&target_file, resource_id, None).unwrap();

    let size_after_remove = fs::metadata(&target_file).unwrap().len();
    let configs_after_remove = find_resources_config(&target_file, |_pos, _config| {}).unwrap();
    assert_eq!(configs_after_remove.len(), 0);

    println!("  ✓ 删除成功");
    println!("  文件大小: {} -> {} 字节 (-{})", size_before_remove, size_after_remove, size_before_remove - size_after_remove);
    println!("  ✓ 验证通过：文件中已无资源");

    // 步骤 6: 清理测试目录
    println!("\n=== 步骤 6: 清理 ===");
    fs::remove_dir_all(&test_dir).unwrap();
    println!("  ✓ 测试目录已删除");
    println!("\n所有测试通过!");
}
