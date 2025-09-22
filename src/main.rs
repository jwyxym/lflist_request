use anyhow::{Error, Result};
use reqwest;
use scraper::{Html, Selector, ElementRef};
use std::{i8};
use urlencoding::{encode};
use std::io::{BufRead};
use std::io::Write;
struct LflistType<'t> {
    id: &'t str,
    write: &'t str,
    ct: i8
}

struct Card {
    name: String,
    code: String
}


#[tokio::main]
async fn main() -> Result<()> {
    let mut args: Vec<String> = std::env::args().collect();
    while args.len() < 5 {
        args.push(String::new());
    }
    if args[1].is_empty() {
        println!("请输入OT:");
        std::io::stdin().read_line(&mut args[1]).expect("");
    }
    if args[2].is_empty() {
        println!("请输入年份:");
        std::io::stdin().read_line(&mut args[1]).expect("");
    }
    if args[3].is_empty() {
        println!("请输入月份（1、4、7、10）:");
        std::io::stdin().read_line(&mut args[2]).expect("");
    }
    if args[4].is_empty() && args[1].trim() == "TCG" {
        println!("请输入日期:");
        std::io::stdin().read_line(&mut args[2]).expect("");
    }
    let ot: &str = args[1].trim();
    let year: &str = args[2].trim();
    let month: &str = args[3].trim();
    let date: &str = args[4].trim();
    let mut lines: Vec<String> = Vec::new();
    if ot == "OCG" {
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
        lines.push(format!("!{}.{}", year, month).to_string());
        if !std::fs::metadata("lflist.conf").is_ok() {
            std::fs::write("lflist.conf", "")?;
        }
        for i in &vec {
            lines.push(format!("{}", i.write));
            if let Some(element) = body.select(&Selector::parse(i.id).unwrap()).next() {
                for td in element.select(&Selector::parse(".cell-ocg").unwrap()) {
                    let name: String = td.text().collect::<String>();
                    let code: String = find_code(&name, 0).await?.name;
                    println!("{} {} --{}", code, i.ct, name);
                    lines.push(format!("{} {} --{}", code, i.ct, name));
                }
            }
        }
        lines.push("".to_string());
    } else {
        let url: String = format!("https://www.yugioh-card.com/en/limited/list_{}-{}-{}/", year, format!("{:0>2}", month), date);
        let response: reqwest::Response = reqwest::get(url).await?;
        let body: String = response.text().await?;
        let body: Html = Html::parse_document(&body);
        lines.push(format!("!{}.{}", year, month).to_string());
        if !std::fs::metadata("lflist.conf").is_ok() {
            std::fs::write("lflist.conf", "")?;
        }
        let mut i = 0;
        let _write = ["#forbidden", "#limit", "#semi limit"];
        let select = Selector::parse(".cardlist").unwrap();
        let tables: Vec<ElementRef<'_>> = body.select(&select).collect();
        for table in tables {
            if i >= 3 {
                break;
            }
            lines.push(format!("{}", _write[i]));
            i += 1;
            for tr in table.select(&Selector::parse("tr").unwrap()).skip(1) {
                let select = Selector::parse("td").unwrap();
                let tds = tr.select(&select);
                let mut name: String = "".to_string();
                let mut forbbiden: String = "".to_string();
                let mut j = 0;
                for td in tds {
                    let text = td.text().collect::<String>();
                    if j == 1 {
                        name = text;
                    } else if j == 2 {
                        forbbiden = text;
                    }
                    j += 1;
                }
                let ct: i8 = find_ct(forbbiden)?;
                let find: Card = find_code(&name, 1).await?;
                let code: String = find.code;
                let name: String = find.name;
                println!("{} {} --{}", code, ct, name);
                if ct < 3 {
                    lines.push(format!("{} {} --{}", code, ct, name));
                }
            }
        }
        lines.push("".to_string());
    }
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

fn find_ct (forbidden: String) ->  Result<i8> {
    match forbidden.as_str() {
        "Forbidden" => {
            Ok(0)
        }
        "Limited" => {
            Ok(1)
        }
        "Semi-Limited" => {
            Ok(2)
        }
        _ => Ok(3)
    }
}

async fn find_code (name: &str, ot: usize) ->  Result<Card, Error> {
    let file: std::fs::File = std::fs::File::open("lflist.conf")?;
    let reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(file);
    for line in reader.lines().skip(1) {
        let line: String = line?;
        if line.contains(name) {
            let code: Vec<&str> = line.split(" ").collect();
            return Ok(Card {
                name: code[2].to_string(),
                code: code[0].to_string()
            });
        }
    }
    let name: String = name.replace("–", "");
    let url: String = format!("https://ygocdb.com/?search={}", encode(&name));
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
                if let Some(element) = body.select(&Selector::parse(".names").unwrap()).next() {
                    let seletor = Selector::parse("h3").unwrap();
                    let mut names = element.select(&seletor).into_iter();
                    let name = names.nth(ot).expect("没有禁限").text().collect::<String>();
                    return Ok(Card {
                        name: name,
                        code: code
                    });
                }
            }
        }
    }
    Ok(Card {
        name: "".to_string(),
        code: "".to_string()
    })
}