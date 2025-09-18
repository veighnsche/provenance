use anyhow::Result;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShieldsBadge {
    pub schemaVersion: u8,
    pub label: String,
    pub message: String,
    pub color: String,
}

impl ShieldsBadge {
    pub fn new<L: Into<String>, M: Into<String>>(label: L, message: M, color: &str) -> Self {
        Self { schemaVersion: 1, label: label.into(), message: message.into(), color: color.to_string() }
    }
}

pub fn badge_error<L: Into<String>, M: Into<String>>(label: L, message: M) -> ShieldsBadge {
    ShieldsBadge::new(label, message, "red")
}

pub fn badge_provenance(verified: bool) -> ShieldsBadge {
    if verified {
        ShieldsBadge::new("provenance", "verified", "brightgreen")
    } else {
        ShieldsBadge::new("provenance", "unverified", "red")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary { pub total: u64, pub passed: u64, pub failed: u64, pub duration_seconds: f64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageTotal { pub pct: f64 }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coverage { pub total: Option<CoverageTotal> }

pub fn badge_tests(summary: &TestSummary) -> ShieldsBadge {
    let message = format!("{}/{} passed", summary.passed, summary.total);
    let color = if summary.failed == 0 { "brightgreen" } else if summary.failed <= 2 { "orange" } else { "red" };
    ShieldsBadge::new("tests", message, color)
}

pub fn badge_coverage(cov: &Coverage) -> ShieldsBadge {
    let pct = cov.total.as_ref().map(|t| t.pct).unwrap_or(0.0);
    let message = format!("{:.1}%", pct);
    let color = if pct >= 90.0 { "brightgreen" } else if pct >= 75.0 { "yellow" } else if pct > 0.0 { "orange" } else { "lightgrey" };
    ShieldsBadge::new("coverage", message, color)
}

pub fn to_svg(b: &ShieldsBadge, style: Option<&str>) -> Result<String> {
    // Very minimal SVG generator (fixed width approximation)
    let _style = style.unwrap_or("flat");
    let label_w = (b.label.len() as u32 * 6 + 10).max(40);
    let msg_w = (b.message.len() as u32 * 6 + 10).max(40);
    let width = label_w + msg_w;
    let height = 20u32;
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">\
         <linearGradient id=\"s\" x2=\"0\" y2=\"100%\"><stop offset=\"0\" stop-color=\"#bbb\" stop-opacity=\".1\"/><stop offset=\"1\" stop-opacity=\".1\"/></linearGradient>\
         <rect rx=\"3\" width=\"{}\" height=\"{}\" fill=\"#555\"/>\
         <rect rx=\"3\" x=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"/>\
         <rect rx=\"3\" width=\"{}\" height=\"{}\" fill=\"url(#s)\"/>\
         <g fill=\"#fff\" text-anchor=\"middle\" font-family=\"DejaVu Sans,Verdana,Geneva,sans-serif\" font-size=\"11\">\
           <text x=\"{}\" y=\"14\">{}</text>\
           <text x=\"{}\" y=\"14\">{}</text>\
         </g>\
        </svg>",
        width, height, width, height, label_w, msg_w, height, svg_color(&b.color), width, height, label_w/2, escape(&b.label), label_w + msg_w/2, escape(&b.message)
    );
    Ok(svg)
}

fn svg_color(name: &str) -> &str {
    match name {
        "brightgreen" => "#4c1",
        "green" => "#97CA00",
        "yellow" => "#dfb317",
        "orange" => "#fe7d37",
        "red" => "#e05d44",
        "blue" => "#007ec6",
        "lightgrey" | "lightgray" => "#9f9f9f",
        other => other,
    }
}

fn escape(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;") }
