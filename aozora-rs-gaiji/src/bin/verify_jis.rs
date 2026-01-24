use aozora_rs_gaiji::JISCharactor;

fn main() {
    println!("=== JIS X 0213 変換検証 ===");

    verify(1, 4, 1, "ぁ");
    check("第3水準", 1, 84, 77);
    verify(1, 1, 1, "\u{3000}");
    verify(2, 94, 86, "\u{2A6B2}");
    verify(2, 94, 85, "\u{9F75}");

    println!("=== 検証完了 ===");
}

fn verify(face: u16, area: u16, point: u16, expected: &str) {
    let result = JISCharactor::new(3, face, area, point).and_then(|c| c.to_char());

    match result {
        Some(c) => {
            if c == expected {
                println!("変換成功：{}-{}-{} -> {}", face, area, point, c);
            } else {
                println!(
                    "変換失敗： {}-{}-{} -> {} ({}になるはずでした)",
                    face, area, point, c, expected
                );
            }
        }
        None => {
            println!("変換失敗 {}-{}-{} -> ×", face, area, point);
        }
    }
}

fn check(name: &str, face: u16, area: u16, point: u16) {
    let result = JISCharactor::new(3, face, area, point).and_then(|c| c.to_char());

    match result {
        Some(c) => println!("{} ({}-{}-{}) -> {}", name, face, area, point, c),
        None => println!("{} ({}-{}-{}) -> None", name, face, area, point),
    }
}
