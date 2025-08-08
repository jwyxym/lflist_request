use anyhow::{Result};
use reqwest;
use scraper::{Html, Selector};
use std::{i8};
use urlencoding::{encode};
use std::io::{BufRead};
use std::io::Write;
struct LflistType<'t> {
    id: &'t str,
    write: &'t str,
    ct: i8
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args: Vec<String> = std::env::args().collect();
    while args.len() < 3 {
        args.push(String::new());
    }
    if args[1].is_empty() {
        println!("请输入年份:");
        std::io::stdin().read_line(&mut args[1]).expect("");
    }
    if args[2].is_empty() {
        println!("请输入月份（1、4、7、10）:");
        std::io::stdin().read_line(&mut args[2]).expect("");
    }
    let year: &str = args[1].trim();
    let month: &str = args[2].trim();
    let url: String = format!("https://www.yugioh-card.com/japan/event/limitregulation/index.php?list={}{}", year, format!("{:0>2}", month));
    let response: reqwest::Response = reqwest::get(url).await?;
    let body: String = response.text().await?;
    let vec: Vec<&LflistType> = vec![
        &LflistType {
            id: "#forbidden",
            write: "#forbidden",
            ct: 0,
        },
        &LflistType {
            id: "#semilimited",
            write: "#limit",
            ct: 1,
        },
        &LflistType {
            id: "#limited",
            write: "#semi limit",
            ct: 2,
        },
    ];
    let body: Html = Html::parse_document(&body);
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("!{}.{}", year, month).to_string());
    if !std::fs::metadata("lflist.conf").is_ok() {
        std::fs::write("lflist.conf", "")?;
    }
    for i in &vec {
        lines.push(format!("{}", i.write));
        if let Some(element) = body.select(&Selector::parse(i.id).unwrap()).next() {
            for td in element.select(&Selector::parse(".cell-ocg").unwrap()) {
                let name: String = td.text().collect::<String>();
                let code: String = find_code(&name).await?;
                println!("{} {} --{}", code, i.ct, name);
                lines.push(format!("{} {} --{}", code, i.ct, name));
            }
        }
    }
    lines.push("".to_string());
    let file: std::fs::File = std::fs::File::open("lflist.conf")?;
    let reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(file);
    let mut chk: bool = false;
    for (i, line) in reader.lines().enumerate() {
        let line: String = line?;
        if i == 0 && line.starts_with("#") {
            lines.insert(0, format!("#[{}.{}]{}", year, month, line.replace("#", "")));
            chk = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !chk {
        lines.insert(0, format!("#[{}.{}]", year, month));
    }
    let mut file: std::fs::File = std::fs::OpenOptions::new().write(true).truncate(true).create(true).open("lflist.conf")?;
    for i in lines {
        writeln!(file, "{}", i)?;
    }
    return Ok(())
}

async fn find_code (name: &str) ->  Result<String> {
    let file: std::fs::File = std::fs::File::open("lflist.conf")?;
    let reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(file);
    for line in reader.lines().skip(1) {
        let line: String = line?;
        if line.contains(name) {
            let code: &str = line.split(" ").next().unwrap();
            return Ok(code.to_string());
        }
    }
    let url: String = format!("https://ygocdb.com/?search={}", encode(name));
    let response: reqwest::Response = reqwest::get(url).await?;
    let body: String = response.text().await?;
    let body: Html = Html::parse_document(&body);
    if let Some(element) = body.select(&Selector::parse(".cardimg").unwrap()).next() {
        if let Some(element) = element.select(&Selector::parse("a").unwrap()).next() {
            if let Some(href) = element.value().attr("href") {
                let mut code: String = href.to_string().split('/').last().unwrap_or("").to_string();
                if (1..8).contains(&code.len()) {
                    code = format!("{:0>8}", code);
                }
                return Ok(code);
            }
        }
    }
    return Ok("".to_string())
}