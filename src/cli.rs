use std::fmt;

use clap::Parser;

/// CLI 固有のエラー型。
///
/// ライブラリの `Jbig2Error` に CLI 固有のエラーを混入させないため、
/// 独立した enum として定義する。
#[derive(Debug)]
pub enum CliError {
    /// 引数バリデーションエラー
    InvalidArgs(String),
    /// 未実装機能（-S 等）
    NotImplemented(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::InvalidArgs(msg) => write!(f, "invalid arguments: {msg}"),
            CliError::NotImplemented(msg) => write!(f, "not implemented: {msg}"),
        }
    }
}

impl std::error::Error for CliError {}

/// JBIG2 encoder — Rust port of jbig2enc.
///
/// C++ 版 `jbig2.cc` の `main()` 引数解析に対応。
#[derive(Parser, Debug)]
#[command(name = "jbig2", about = "JBIG2 encoder - Rust port of jbig2enc")]
pub struct Args {
    /// Output file basename for symbol mode
    #[arg(short = 'b', default_value = "output")]
    pub basename: String,

    /// Use TPGD duplicate line removal in generic region coder
    #[arg(short = 'd', long = "duplicate-line-removal")]
    pub duplicate_line_removal: bool,

    /// Produce PDF ready data
    #[arg(short = 'p', long = "pdf")]
    pub pdf: bool,

    /// Use text region, not generic coder
    #[arg(short = 's', long = "symbol-mode")]
    pub symbol_mode: bool,

    /// Set classification threshold for symbol coder (0.4–0.97)
    #[arg(short = 't', default_value_t = 0.92)]
    pub threshold: f32,

    /// Set classification weight for symbol coder (0.1–0.9)
    #[arg(short = 'w', default_value_t = 0.5)]
    pub weight: f32,

    /// Set 1 bpp threshold (0–255)
    #[arg(short = 'T')]
    pub bw_threshold: Option<u8>,

    /// Use global BW threshold on 8 bpp images (default: adaptive)
    #[arg(short = 'G', long = "global")]
    pub global: bool,

    /// Use refinement (requires -s: lossless)
    #[arg(short = 'r', long = "refine")]
    pub refine: bool,

    /// Dump thresholded image as PNG
    #[arg(short = 'O')]
    pub output_threshold: Option<String>,

    /// Upsample 2x before thresholding
    #[arg(short = '2')]
    pub up2: bool,

    /// Upsample 4x before thresholding
    #[arg(short = '4')]
    pub up4: bool,

    /// Remove images from mixed input and save separately
    #[arg(short = 'S')]
    pub segment: bool,

    /// Write images from mixed input as JPEG (-S dependent)
    #[arg(short = 'j', long = "jpeg-output")]
    pub jpeg_output: bool,

    /// Use automatic thresholding in symbol encoder
    #[arg(short = 'a', long = "auto-thresh")]
    pub auto_thresh: bool,

    /// Disable use of hash function for automatic thresholding
    #[arg(long = "no-hash")]
    pub no_hash: bool,

    /// Force DPI (1–9600)
    #[arg(short = 'D', long = "dpi")]
    pub dpi: Option<u32>,

    /// Be verbose
    #[arg(short = 'v')]
    pub verbose: bool,

    /// Input files
    #[arg(required = true)]
    pub files: Vec<String>,
}

impl Args {
    /// 引数のバリデーションを実行する。
    ///
    /// clap によるパース後、値の範囲や組み合わせの整合性を検証する。
    pub fn validate(&self) -> Result<(), CliError> {
        if self.refine {
            if !self.symbol_mode {
                return Err(CliError::InvalidArgs(
                    "refinement requires symbol mode (-s)".into(),
                ));
            }
            return Err(CliError::InvalidArgs(
                "refinement broke in recent releases since it's rarely used".into(),
            ));
        }

        if self.up2 && self.up4 {
            return Err(CliError::InvalidArgs("cannot use both -2 and -4".into()));
        }

        if !(0.4..=0.97).contains(&self.threshold) {
            return Err(CliError::InvalidArgs(format!(
                "threshold must be between 0.40 and 0.97, got {}",
                self.threshold
            )));
        }

        if !(0.1..=0.9).contains(&self.weight) {
            return Err(CliError::InvalidArgs(format!(
                "weight must be between 0.10 and 0.90, got {}",
                self.weight
            )));
        }

        if let Some(dpi) = self.dpi
            && !(1..=9600).contains(&dpi)
        {
            return Err(CliError::InvalidArgs(format!(
                "DPI must be between 1 and 9600, got {dpi}"
            )));
        }

        if self.segment {
            return Err(CliError::NotImplemented(
                "text/graphics segmentation (-S) is not yet implemented".into(),
            ));
        }

        Ok(())
    }

    /// 二値化閾値の実効値を返す。
    ///
    /// `-T` で明示指定されていればその値、なければ:
    /// - `-G`（グローバル）モード: 128
    /// - デフォルト（適応的）モード: 200
    #[allow(dead_code)] // PR 2 の pipeline で使用予定
    pub fn effective_bw_threshold(&self) -> u8 {
        self.bw_threshold
            .unwrap_or(if self.global { 128 } else { 200 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_args() -> Args {
        Args {
            basename: "output".to_string(),
            duplicate_line_removal: false,
            pdf: false,
            symbol_mode: false,
            threshold: 0.92,
            weight: 0.5,
            bw_threshold: None,
            global: false,
            refine: false,
            output_threshold: None,
            up2: false,
            up4: false,
            segment: false,
            jpeg_output: false,
            auto_thresh: false,
            no_hash: false,
            dpi: None,
            verbose: false,
            files: vec!["input.png".to_string()],
        }
    }

    // --- デフォルト値検証 ---

    #[test]
    fn default_args_pass_validation() {
        let args = default_args();
        assert!(args.validate().is_ok());
    }

    // --- bw_threshold デフォルト値 ---

    #[test]
    fn default_bw_threshold_is_200() {
        let args = default_args();
        assert_eq!(args.effective_bw_threshold(), 200);
    }

    #[test]
    fn global_mode_bw_threshold_is_128() {
        let args = Args {
            global: true,
            ..default_args()
        };
        assert_eq!(args.effective_bw_threshold(), 128);
    }

    #[test]
    fn explicit_bw_threshold_overrides_default() {
        let args = Args {
            bw_threshold: Some(150),
            ..default_args()
        };
        assert_eq!(args.effective_bw_threshold(), 150);
    }

    #[test]
    fn explicit_bw_threshold_overrides_global_default() {
        let args = Args {
            bw_threshold: Some(150),
            global: true,
            ..default_args()
        };
        assert_eq!(args.effective_bw_threshold(), 150);
    }

    // --- threshold 範囲 ---

    #[test]
    fn threshold_at_min_boundary_is_accepted() {
        let args = Args {
            threshold: 0.4,
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn threshold_at_max_boundary_is_accepted() {
        let args = Args {
            threshold: 0.97,
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn threshold_below_min_is_rejected() {
        let args = Args {
            threshold: 0.39,
            ..default_args()
        };
        let err = args.validate().unwrap_err();
        assert!(matches!(err, CliError::InvalidArgs(_)));
    }

    #[test]
    fn threshold_above_max_is_rejected() {
        let args = Args {
            threshold: 0.98,
            ..default_args()
        };
        let err = args.validate().unwrap_err();
        assert!(matches!(err, CliError::InvalidArgs(_)));
    }

    // --- weight 範囲 ---

    #[test]
    fn weight_at_min_boundary_is_accepted() {
        let args = Args {
            weight: 0.1,
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn weight_at_max_boundary_is_accepted() {
        let args = Args {
            weight: 0.9,
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn weight_below_min_is_rejected() {
        let args = Args {
            weight: 0.09,
            ..default_args()
        };
        let err = args.validate().unwrap_err();
        assert!(matches!(err, CliError::InvalidArgs(_)));
    }

    #[test]
    fn weight_above_max_is_rejected() {
        let args = Args {
            weight: 0.91,
            ..default_args()
        };
        let err = args.validate().unwrap_err();
        assert!(matches!(err, CliError::InvalidArgs(_)));
    }

    // --- -2/-4 排他 ---

    #[test]
    fn up2_and_up4_are_exclusive() {
        let args = Args {
            up2: true,
            up4: true,
            ..default_args()
        };
        let err = args.validate().unwrap_err();
        assert!(matches!(err, CliError::InvalidArgs(_)));
    }

    #[test]
    fn up2_alone_is_accepted() {
        let args = Args {
            up2: true,
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn up4_alone_is_accepted() {
        let args = Args {
            up4: true,
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    // --- -r は -s 必須（かつ broken） ---

    #[test]
    fn refine_without_symbol_mode_is_rejected() {
        let args = Args {
            refine: true,
            symbol_mode: false,
            ..default_args()
        };
        let err = args.validate().unwrap_err();
        assert!(matches!(err, CliError::InvalidArgs(_)));
    }

    #[test]
    fn refine_with_symbol_mode_is_rejected_as_broken() {
        let args = Args {
            refine: true,
            symbol_mode: true,
            ..default_args()
        };
        let err = args.validate().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("broke"), "expected 'broken' error, got: {msg}");
    }

    // --- DPI 範囲 ---

    #[test]
    fn dpi_at_min_boundary_is_accepted() {
        let args = Args {
            dpi: Some(1),
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn dpi_at_max_boundary_is_accepted() {
        let args = Args {
            dpi: Some(9600),
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn dpi_zero_is_rejected() {
        let args = Args {
            dpi: Some(0),
            ..default_args()
        };
        let err = args.validate().unwrap_err();
        assert!(matches!(err, CliError::InvalidArgs(_)));
    }

    #[test]
    fn dpi_above_max_is_rejected() {
        let args = Args {
            dpi: Some(9601),
            ..default_args()
        };
        let err = args.validate().unwrap_err();
        assert!(matches!(err, CliError::InvalidArgs(_)));
    }

    // --- -S 未実装 ---

    #[test]
    fn segment_flag_is_not_implemented() {
        let args = Args {
            segment: true,
            ..default_args()
        };
        let err = args.validate().unwrap_err();
        assert!(matches!(err, CliError::NotImplemented(_)));
    }

    // --- clap パース統合テスト ---

    #[test]
    fn clap_parse_defaults() {
        let args = Args::parse_from(["jbig2", "input.png"]);
        assert_eq!(args.basename, "output");
        assert!(!args.duplicate_line_removal);
        assert!(!args.pdf);
        assert!(!args.symbol_mode);
        assert_eq!(args.threshold, 0.92);
        assert_eq!(args.weight, 0.5);
        assert_eq!(args.bw_threshold, None);
        assert!(!args.global);
        assert!(!args.refine);
        assert!(args.output_threshold.is_none());
        assert!(!args.up2);
        assert!(!args.up4);
        assert!(!args.segment);
        assert!(!args.jpeg_output);
        assert!(!args.auto_thresh);
        assert!(!args.no_hash);
        assert!(args.dpi.is_none());
        assert!(!args.verbose);
        assert_eq!(args.files, vec!["input.png"]);
    }

    #[test]
    fn clap_parse_all_flags() {
        let args = Args::parse_from([
            "jbig2",
            "-b",
            "out",
            "-d",
            "-p",
            "-s",
            "-t",
            "0.85",
            "-w",
            "0.3",
            "-T",
            "180",
            "-G",
            "-O",
            "debug.png",
            "-2",
            "-a",
            "--no-hash",
            "-D",
            "300",
            "-v",
            "a.png",
            "b.png",
        ]);
        assert_eq!(args.basename, "out");
        assert!(args.duplicate_line_removal);
        assert!(args.pdf);
        assert!(args.symbol_mode);
        assert_eq!(args.threshold, 0.85);
        assert_eq!(args.weight, 0.3);
        assert_eq!(args.bw_threshold, Some(180));
        assert!(args.global);
        assert_eq!(args.output_threshold.as_deref(), Some("debug.png"));
        assert!(args.up2);
        assert!(!args.up4);
        assert!(args.auto_thresh);
        assert!(args.no_hash);
        assert_eq!(args.dpi, Some(300));
        assert!(args.verbose);
        assert_eq!(args.files, vec!["a.png", "b.png"]);
    }
}
