use crate::cube::Move;

/// 操作履歴を管理する構造体
#[derive(Debug, Clone)]
pub struct History {
    /// Undo用スタック（実行済み操作）
    undo_stack: Vec<Move>,

    /// Redo用スタック（undoされた操作）
    redo_stack: Vec<Move>,

    /// 履歴の最大サイズ
    max_size: usize,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    /// デフォルトサイズ（100件）で新しい履歴を作成
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    /// 指定サイズで新しい履歴を作成
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size,
        }
    }

    /// 操作を履歴に追加
    pub fn push(&mut self, mv: Move) {
        self.undo_stack.push(mv);

        // 最大サイズを超えたら古い操作を削除
        if self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }

        // 新しい操作が追加されたらredoスタックをクリア
        self.redo_stack.clear();
    }

    /// Undo: 最後の操作を取り消す
    /// 戻り値: 取り消す操作の逆操作
    pub fn undo(&mut self) -> Option<Move> {
        if let Some(mv) = self.undo_stack.pop() {
            self.redo_stack.push(mv);
            Some(mv.inverse())
        } else {
            None
        }
    }

    /// Redo: 取り消した操作をやり直す
    /// 戻り値: やり直す操作
    pub fn redo(&mut self) -> Option<Move> {
        if let Some(mv) = self.redo_stack.pop() {
            self.undo_stack.push(mv);
            Some(mv)
        } else {
            None
        }
    }

    /// Undoが可能かどうか
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Redoが可能かどうか
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// 履歴をクリア
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Undo可能な操作数
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Redo可能な操作数
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_undo() {
        let mut history = History::new();
        history.push(Move::R);
        history.push(Move::U);

        assert_eq!(history.undo_count(), 2);
        assert_eq!(history.undo(), Some(Move::Up)); // U の逆操作
        assert_eq!(history.undo(), Some(Move::Rp)); // R の逆操作
        assert_eq!(history.undo(), None);
    }

    #[test]
    fn test_redo() {
        let mut history = History::new();
        history.push(Move::R);
        history.undo();

        assert_eq!(history.redo(), Some(Move::R));
        assert_eq!(history.redo(), None);
    }

    #[test]
    fn test_clear_redo_on_new_push() {
        let mut history = History::new();
        history.push(Move::R);
        history.undo();
        assert!(history.can_redo());

        history.push(Move::U);
        assert!(!history.can_redo());
    }
}
