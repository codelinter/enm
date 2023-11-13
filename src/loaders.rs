use std::io::Read;

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use reqwest::blocking::Response;

pub struct ResponseProgress {
    loaders: Option<ProgressBar>,
    response: Response,
}

#[derive(Default, Clone, Debug, clap::ValueEnum)]
pub enum ProgressConfig {
    #[default]
    Auto,
    Never,
    Always,
}

impl ProgressConfig {
    pub fn enabled(&self, config: &crate::app_config::AppConfig) -> bool {
        match self {
            Self::Never => false,
            Self::Always => true,
            Self::Auto => config
                .ll_int()
                .is_writable(crate::ll_int::LLInt::Info),
        }
    }
}

fn make_loadanimate_bar(size: u64, target: ProgressDrawTarget) -> ProgressBar {
    let bar = ProgressBar::with_draw_target(Some(size), target);

    bar.set_style(
        ProgressStyle::with_template(
            "{elapsed_precise:.white.dim} {wide_bar:.green} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
        )
        .unwrap()
        .progress_chars("=== == =▏  "),
    );

    bar
}

impl ResponseProgress {
    pub fn new(response: Response, target: ProgressDrawTarget) -> Self {
        Self {
            loaders: response
                .content_length()
                .map(|len| make_loadanimate_bar(len, target)),
            response,
        }
    }

    pub fn finish(&self) {
        if let Some(ref bar) = self.loaders {
            bar.finish();
        }
    }
}

impl Read for ResponseProgress {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let size = self.response.read(buf)?;

        if let Some(ref bar) = self.loaders {
            bar.inc(size as u64);
        }

        Ok(size)
    }
}

impl Drop for ResponseProgress {
    fn drop(&mut self) {
        self.finish();
        eprintln!();
    }
}
