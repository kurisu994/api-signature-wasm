use std::fs;
use std::path::Path;
use serde_json::{Value, json};

fn main() {
    let package_json_path = Path::new("pkg/package.json");

    // 检查 package.json 是否存在
    if package_json_path.exists() {
        // 读取 package.json 文件
        let data = fs::read_to_string(package_json_path)
            .expect("Unable to read package.json");
        
        // 解析 JSON 数据
        let mut json_data: Value = serde_json::from_str(&data)
            .expect("Unable to parse package.json");

        // 添加/修改自定义字段
        json_data["description"] = json!("This is a custom Rust WASM package");
        json_data["repository"] = json!({
            "type": "git",
            "url": "https://github.com/your-repo.git"
        });
        json_data["author"] = json!("Your Name <your-email@example.com>");
        json_data["license"] = json!("MIT");

        // 将修改后的内容写回 package.json
        let new_data = serde_json::to_string_pretty(&json_data)
            .expect("Unable to serialize modified package.json");

        fs::write(package_json_path, new_data)
            .expect("Unable to write package.json");
    } else {
        println!("package.json not found");
    }
}