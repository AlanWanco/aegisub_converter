use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn aegisub_uue_decode(s: &str) -> String {
    let bytes: Vec<u8> = s.as_bytes().iter().map(|&b| b.saturating_sub(33)).collect();
    let mut res = Vec::new();

    for chunk in bytes.chunks(4) {
        if chunk.len() >= 2 {
            res.push((chunk[0] << 2) | (chunk[1] >> 4));
        }
        if chunk.len() >= 3 {
            res.push(((chunk[1] & 0x0F) << 4) | (chunk[2] >> 2));
        }
        if chunk.len() >= 4 {
            res.push(((chunk[2] & 0x03) << 6) | chunk[3]);
        }
    }

    // 尝试 UTF-8 解码
    let decoded = String::from_utf8_lossy(&res).to_string();
    // 去掉解码后文字中的换行符
    decoded.replace(['\n', '\r'], "")
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("用法: 请将 .ass 文件拖到此程序图标上，或在命令行输入文件路径。");
        println!("\n按回车键退出...");
        let _ = io::stdin().read_line(&mut String::new());
        return Ok(());
    }

    let input_path = &args[1];
    let path = Path::new(input_path);
    if !path.exists() {
        println!("错误: 文件不存在: {}", input_path);
        return Ok(());
    }

    let file_content = fs::read(input_path)?;
    // 处理 UTF-8 BOM
    let content = if file_content.starts_with(&[0xEF, 0xBB, 0xBF]) {
        String::from_utf8_lossy(&file_content[3..])
    } else {
        String::from_utf8_lossy(&file_content)
    };

    let mut extradata = HashMap::new();
    let mut in_extradata = false;
    let mut in_events = false;
    let mut new_lines = Vec::new();

    let re_data = Regex::new(r"Data:\s*(\d+),stt,([ue])(.*)").unwrap();
    let re_placeholder = Regex::new(r"\{=(\d+)\}").unwrap();

    // 第一次遍历：解析 Extradata
    for line in content.lines() {
        let stripped = line.trim();
        if stripped == "[Aegisub Extradata]" {
            in_extradata = true;
            continue;
        }
        if in_extradata {
            if stripped.starts_with('[') {
                in_extradata = false;
            } else if let Some(caps) = re_data.captures(stripped) {
                let id = caps.get(1).unwrap().as_str().to_string();
                let dtype = caps.get(2).unwrap().as_str();
                let data = caps.get(3).unwrap().as_str();

                let decoded = if dtype == "e" {
                    data.replace(['\n', '\r'], "")
                } else {
                    aegisub_uue_decode(data)
                };
                extradata.insert(id, decoded);
            }
        }
    }

    // 第二次遍历：替换内容，并过滤掉 Extradata 部分
    let mut skip_output = false;
    for line in content.lines() {
        let trimmed = line.trim();

        // 如果遇到 Extradata 节，开始停止输出
        if trimmed == "[Aegisub Extradata]" {
            skip_output = true;
            continue;
        }

        // 如果在跳过状态下遇到了下一个节头，则恢复
        if skip_output && trimmed.starts_with('[') && trimmed != "[Aegisub Extradata]" {
            skip_output = false;
        }

        if skip_output {
            continue;
        }

        if trimmed == "[Events]" {
            in_events = true;
            new_lines.push(line.to_string());
            continue;
        }

        if in_events {
            if trimmed.starts_with('[') {
                in_events = false;
            } else if line.starts_with("Dialogue:") {
                let parts: Vec<&str> = line.splitn(10, ',').collect();
                if parts.len() == 10 {
                    let text = parts[9];

                    let ids: Vec<String> = re_placeholder
                        .captures_iter(text)
                        .map(|c| c.get(1).unwrap().as_str().to_string())
                        .collect();

                    if !ids.is_empty() {
                        let raw_content = re_placeholder.replace_all(text, "").trim().to_string();
                        let decrypted_parts: Vec<String> = ids
                            .iter()
                            .map(|id| {
                                extradata
                                    .get(id)
                                    .cloned()
                                    .unwrap_or_else(|| format!("{{={}}}", id))
                            })
                            .collect();
                        let decrypted_text = decrypted_parts.join("\\N");

                        let final_text = if !raw_content.is_empty() {
                            format!("[原文]{}\\N{}", raw_content, decrypted_text)
                        } else {
                            decrypted_text
                        };

                        let mut new_line = parts[..9].join(",");
                        new_line.push(',');
                        new_line.push_str(&final_text);
                        new_lines.push(new_line);
                        continue;
                    }
                }
            }
        }
        new_lines.push(line.to_string());
    }

    let stem = path.file_stem().unwrap().to_str().unwrap();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let output_path = parent.join(format!("{}_converted.ass", stem));

    let mut output_file = fs::File::create(&output_path)?;
    output_file.write_all(&[0xEF, 0xBB, 0xBF])?; // BOM
    for line in new_lines {
        writeln!(output_file, "{}", line)?;
    }

    println!("成功！转换后的文件已生成，且不包含 Extradata 部分。");
    println!("转换文件: {:?}", output_path);
    println!("\n按回车键退出...");
    let _ = io::stdin().read_line(&mut String::new());

    Ok(())
}
