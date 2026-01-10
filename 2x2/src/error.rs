use std::io;

/// アプリケーション全体で使用するエラー型
#[derive(Debug, thiserror::Error)]
pub enum CubeError {
    /// 無効な色配列
    #[error("無効な色配列: {0}")]
    InvalidColors(String),

    /// 無効な色文字
    #[error("無効な色文字: {0}")]
    InvalidColorChar(char),

    /// ファイルフォーマットエラー
    #[error("ファイルフォーマットエラー: {0}")]
    InvalidFormat(String),

    /// 色が見つからない
    #[error("{0}が見つかりません")]
    ColorNotFound(String),

    /// 向き復元失敗
    #[error("向き復元に失敗: {0}")]
    OrientationRestoreFailed(String),

    /// 内部エラー（通常は発生しない）
    #[error("内部エラー: {0}")]
    Internal(String),

    /// ファイルI/Oエラー
    #[error("ファイル操作エラー: {0}")]
    Io(#[from] io::Error),

    /// 無効なキューブ状態
    #[error("無効なキューブ状態: {0}")]
    InvalidState(String),

    /// コーナーパリティエラー
    #[error("コーナーパリティエラー: {0}")]
    CornerParity(String),
}

/// Result型のエイリアス
pub type Result<T> = std::result::Result<T, CubeError>;
