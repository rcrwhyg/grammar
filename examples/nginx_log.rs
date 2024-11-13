// parse:
// 93.180.71.3 - - [17/May/2015:08:05:32 +0000] "GET /downloads/product_1 HTTP/1.1" 304 0 "-" "Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)"
// https://raw.githubusercontent.com/elastic/examples/master/Common%20Data%20Formats/nginx_logs/nginx_logs

use anyhow::Result;
use regex::Regex;

#[allow(unused)]
#[derive(Debug)]
struct NginxLog {
    addr: String,
    date_time: String,
    method: String,
    url: String,
    protocol: String,
    status: u16,
    body_bytes: u64,
    referer: String,
    user_agent: String,
}

fn main() -> Result<()> {
    let s = r#"93.180.71.3 - - [17/May/2015:08:05:32 +0000] "GET /downloads/product_1 HTTP/1.1" 304 0 "-" "Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)""#;
    let log = parse_nginx_log(s)?;
    println!("{:?}", log);

    Ok(())
}

fn parse_nginx_log(s: &str) -> Result<NginxLog> {
    let re = Regex::new(
        r#"^(?<ip>\S+)\s+\S+\s+\S+\s+\[(?<date>[^\]]+)\]\s+"(?<method>\S+)\s+(?<url>\S+)\s+(?<protocol>[^"]+)"\s+(?<status>\d+)\s+(?<bytes>\d+)\s+"(?<referer>[^"]+)"\s+"(?<ua>[^"]+)"$"#,
    )?;
    let caps = re.captures(s).ok_or(anyhow::anyhow!("invalid nginx log"))?;

    let addr = caps
        .name("ip")
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow::anyhow!("Parse ip error"))?;
    let date_time = caps
        .name("date")
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow::anyhow!("Parse date error"))?;
    let method = caps
        .name("method")
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow::anyhow!("Parse method error"))?;
    let url = caps
        .name("url")
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow::anyhow!("Parse url error"))?;
    let protocol = caps
        .name("protocol")
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow::anyhow!("Parse protocol error"))?;
    let status = caps
        .name("status")
        .map(|m| m.as_str().parse())
        .ok_or(anyhow::anyhow!("Parse status error"))??;
    let body_bytes = caps
        .name("bytes")
        .map(|m| m.as_str().parse())
        .ok_or(anyhow::anyhow!("Parse bytes error"))??;
    let referer = caps
        .name("referer")
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow::anyhow!("Parse referer error"))?;
    let user_agent = caps
        .name("ua")
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow::anyhow!("Parse user_agent error"))?;

    Ok(NginxLog {
        addr,
        date_time,
        method,
        url,
        protocol,
        status,
        body_bytes,
        referer,
        user_agent,
    })
}
