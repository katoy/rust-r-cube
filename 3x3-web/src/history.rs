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
    /// デフォルトサイズ（100件）で新しい履歴を作成します。
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_3x3::history::History;
    ///
    /// let history = History::new();
    /// assert!(history.can_undo() == false);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    /// 指定サイズで新しい履歴を作成します。
    ///
    /// # 引数
    ///
    /// - `max_size` - 保持する履歴の最大件数
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_3x3::history::History;
    ///
    /// let history = History::with_capacity(50);
    /// ```
    #[must_use]
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size,
        }
    }

    /// 操作を履歴に追加します。
    ///
    /// # 引数
    ///
    /// - `mv` - 追加する操作
    ///
    /// # 動作
    ///
    /// - 操作をUndoスタックに追加
    /// - 最大サイズを超えた場合、最も古い操作を削除
    /// - Redoスタックをクリア
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_3x3::history::History;
    /// use rubiks_cube_3x3::cube::Move;
    ///
    /// let mut history = History::new();
    /// history.push(Move::R);
    /// assert_eq!(history.undo_count(), 1);
    /// ```
    pub fn push(&mut self, mv: Move) {
        self.undo_stack.push(mv);

        // 最大サイズを超えたら古い操作を削除
        if self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }

        // 新しい操作が追加されたらredoスタックをクリア
        self.redo_stack.clear();
    }

    /// 最後の操作を取り消します（Undo）。
    ///
    /// # 戻り値
    ///
    /// - `Some(Move)` - 取り消すための逆操作
    /// - `None` - Undo可能な操作がない
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_3x3::history::History;
    /// use rubiks_cube_3x3::cube::Move;
    ///
    /// let mut history = History::new();
    /// history.push(Move::R);
    /// assert_eq!(history.undo(), Some(Move::Rp)); // Rの逆操作
    /// ```
    pub fn undo(&mut self) -> Option<Move> {
        if let Some(mv) = self.undo_stack.pop() {
            self.redo_stack.push(mv);
            Some(mv.inverse())
        } else {
            None
        }
    }

    /// 取り消した操作をやり直します（Redo）。
    ///
    /// # 戻り値
    ///
    /// - `Some(Move)` - やり直す操作
    /// - `None` - Redo可能な操作がない
    ///
    /// # 例
    ///
    /// ```
    /// use rubiks_cube_3x3::history::History;
    /// use rubiks_cube_3x3::cube::Move;
    ///
    /// let mut history = History::new();
    /// history.push(Move::R);
    /// history.undo();
    /// assert_eq!(history.redo(), Some(Move::R)); // 元の操作
    /// ```
    pub fn redo(&mut self) -> Option<Move> {
        if let Some(mv) = self.redo_stack.pop() {
            self.undo_stack.push(mv);
            Some(mv)
        } else {
            None
        }
    }

    /// Undoが可能かどうかを返します。
    ///
    /// # 戻り値
    ///
    /// - `true` - Undo可能（履歴が1件以上）
    /// - `false` - Undo不可（履歴が空）
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Redoが可能かどうかを返します。
    ///
    /// # 戻り値
    ///
    /// - `true` - Redo可能（Undoした操作が1件以上）
    /// - `false` - Redo不可（Undoした操作がない）
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// 履歴をクリアします。
    ///
    /// UndoスタックとRedoスタックの両方を空にします。
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Undo可能な操作数を返します。
    ///
    /// # 戻り値
    ///
    /// Undoスタックに保持されている操作の数
    #[must_use]
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Redo可能な操作数を返します。
    ///
    /// # 戻り値
    ///
    /// Redoスタックに保持されている操作の数
    #[must_use]
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}
