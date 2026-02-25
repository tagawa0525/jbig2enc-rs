use std::fmt;

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

/// CLI 引数定義。
///
/// C++ 版 `jbig2.cc` の `main()` 引数解析に対応。
#[derive(Debug)]
pub struct Args {
    /// 出力ファイルベース名（-b）
    pub basename: String,
    /// TPGD 重複行除去（-d）
    pub duplicate_line_removal: bool,
    /// PDF フラグメントモード（-p）
    pub pdf: bool,
    /// シンボル/テキストリージョン符号化（-s）
    pub symbol_mode: bool,
    /// シンボル分類閾値（-t, 0.4–0.97）
    pub threshold: f32,
    /// 分類重み（-w, 0.1–0.9）
    pub weight: f32,
    /// 二値化閾値（-T, 0–255）。None の場合はモードに応じたデフォルトを使用。
    pub bw_threshold: Option<u8>,
    /// グローバル閾値（-G）
    pub global: bool,
    /// リファインメント（-r）
    pub refine: bool,
    /// 二値化画像のデバッグ保存先（-O）
    pub output_threshold: Option<String>,
    /// 2x アップサンプリング（-2）
    pub up2: bool,
    /// 4x アップサンプリング（-4）
    pub up4: bool,
    /// テキスト/グラフィクス分離（-S, 未実装）
    pub segment: bool,
    /// 分離画像を JPEG で保存（-j, -S 依存）
    pub jpeg_output: bool,
    /// 自動シンボル閾値（-a）
    pub auto_thresh: bool,
    /// ハッシュ無効化（--no-hash）
    pub no_hash: bool,
    /// DPI 強制（-D, 1–9600）
    pub dpi: Option<u32>,
    /// 詳細出力（-v）
    pub verbose: bool,
    /// 入力ファイル
    pub files: Vec<String>,
}

impl Args {
    /// 引数のバリデーションを実行する。
    ///
    /// clap によるパース後、値の範囲や組み合わせの整合性を検証する。
    pub fn validate(&self) -> Result<(), CliError> {
        todo!()
    }

    /// 二値化閾値の実効値を返す。
    ///
    /// `-T` で明示指定されていればその値、なければ:
    /// - `-G`（グローバル）モード: 128
    /// - デフォルト（適応的）モード: 200
    pub fn effective_bw_threshold(&self) -> u8 {
        todo!()
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
    #[ignore = "not yet implemented"]
    fn default_args_pass_validation() {
        let args = default_args();
        assert!(args.validate().is_ok());
    }

    // --- bw_threshold デフォルト値 ---

    #[test]
    #[ignore = "not yet implemented"]
    fn default_bw_threshold_is_200() {
        let args = default_args();
        assert_eq!(args.effective_bw_threshold(), 200);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn global_mode_bw_threshold_is_128() {
        let args = Args {
            global: true,
            ..default_args()
        };
        assert_eq!(args.effective_bw_threshold(), 128);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn explicit_bw_threshold_overrides_default() {
        let args = Args {
            bw_threshold: Some(150),
            ..default_args()
        };
        assert_eq!(args.effective_bw_threshold(), 150);
    }

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn threshold_at_min_boundary_is_accepted() {
        let args = Args {
            threshold: 0.4,
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn threshold_at_max_boundary_is_accepted() {
        let args = Args {
            threshold: 0.97,
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn threshold_below_min_is_rejected() {
        let args = Args {
            threshold: 0.39,
            ..default_args()
        };
        let err = args.validate().unwrap_err();
        assert!(matches!(err, CliError::InvalidArgs(_)));
    }

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn weight_at_min_boundary_is_accepted() {
        let args = Args {
            weight: 0.1,
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn weight_at_max_boundary_is_accepted() {
        let args = Args {
            weight: 0.9,
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn weight_below_min_is_rejected() {
        let args = Args {
            weight: 0.09,
            ..default_args()
        };
        let err = args.validate().unwrap_err();
        assert!(matches!(err, CliError::InvalidArgs(_)));
    }

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn up2_alone_is_accepted() {
        let args = Args {
            up2: true,
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn up4_alone_is_accepted() {
        let args = Args {
            up4: true,
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    // --- -r は -s 必須（かつ broken） ---

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn dpi_at_min_boundary_is_accepted() {
        let args = Args {
            dpi: Some(1),
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn dpi_at_max_boundary_is_accepted() {
        let args = Args {
            dpi: Some(9600),
            ..default_args()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn dpi_zero_is_rejected() {
        let args = Args {
            dpi: Some(0),
            ..default_args()
        };
        let err = args.validate().unwrap_err();
        assert!(matches!(err, CliError::InvalidArgs(_)));
    }

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn segment_flag_is_not_implemented() {
        let args = Args {
            segment: true,
            ..default_args()
        };
        let err = args.validate().unwrap_err();
        assert!(matches!(err, CliError::NotImplemented(_)));
    }
}
