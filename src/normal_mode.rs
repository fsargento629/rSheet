//! Normal-mode command parser for Vim-style key sequences.
//!
//! The parser is a two-state machine:
//!   * `Idle`            – accumulating an optional count1, waiting for an operator or direct motion
//!   * `OperatorPending` – an operator (d/y) was entered; now accumulating count2 and waiting for a motion
//!
//! Only `KeyCode::Char(c)` events are fed here. Non-printable keys (Esc, arrows,
//! Enter, F-keys, …) must be handled by the caller before consulting this machine.

/// Motion directions used by operator commands and standalone navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    Up,
    Down,
    /// `0` with no preceding count → jump to the first column of the row.
    StartOfRow,
    /// `$` → jump to / operate to the end of the row.
    EndOfRow,
    /// `w` – forward word; treated as one step right in a spreadsheet.
    WordForward,
    /// `b` – back word; treated as one step left in a spreadsheet.
    WordBack,
}

/// Which high-level operation was requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Yank,
}

/// Clipboard content produced by a yank operation.
#[derive(Debug, Clone)]
pub enum ClipboardContent {
    /// A horizontal slice of raw cell values from a single row.
    Cells(Vec<String>),
    /// One or more complete rows of raw cell values.
    Rows(Vec<Vec<String>>),
}

/// A fully-resolved command ready for the application to execute.
#[derive(Debug, Clone)]
pub enum NormalCommand {
    /// Move the cursor using the given motion, repeated `count` times.
    Move { motion: Motion, count: usize },
    /// Apply a delete operation driven by `motion`, with the given effective count.
    Delete { motion: Motion, count: usize },
    /// Delete `count` complete rows starting at the cursor row (`dd`, `2dd`, …).
    DeleteRow { count: usize },
    /// Yank (copy) driven by `motion`.
    Yank { motion: Motion, count: usize },
    /// Yank `count` complete rows starting at the cursor row (`yy`, `2yy`, …).
    YankRow { count: usize },
    /// Paste clipboard content before or after the cursor, repeated `count` times.
    Paste { before: bool, count: usize },
    /// Enter Visual mode (`a` / F2 handled separately for F2).
    EnterVisualMode,
    /// Enter Insert mode, optionally seeding the buffer with `initial`.
    /// * `None`          → loads the current cell's raw content
    /// * `Some("")`      → blank buffer (overwrite)
    /// * `Some("=")`     → formula entry shortcut
    EnterInsertMode(Option<String>),
    /// Save the spreadsheet (`s` / `S`).
    Save,
    /// Quit the application (`q` / `Q`).
    Quit,
    /// Delete the current cell content only (`Delete` / `Backspace` handled by caller;
    /// this variant is not currently produced but reserved for future use).
    #[allow(dead_code)]
    DeleteCell,
    /// The accumulated state was cancelled (Esc, or an unrecognised key in operator-pending).
    Reset,
}

// ---------------------------------------------------------------------------
// Internal sub-state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum SubState {
    Idle,
    OperatorPending,
}

// ---------------------------------------------------------------------------
// Public state machine struct
// ---------------------------------------------------------------------------

/// Holds the in-progress state of a normal-mode key sequence.
///
/// Feed each `KeyCode::Char(c)` through [`NormalModeState::process_char`].
/// Non-char keys must be handled externally; call [`NormalModeState::reset`] as
/// appropriate (e.g. on Esc or arrow key press).
#[derive(Debug)]
pub struct NormalModeState {
    sub_state: SubState,
    /// Digits accumulated before the operator.
    count1: usize,
    /// The pending operator, if any.
    operator: Option<Operator>,
    /// Digits accumulated after the operator.
    count2: usize,
}

impl NormalModeState {
    pub fn new() -> Self {
        Self {
            sub_state: SubState::Idle,
            count1: 0,
            operator: None,
            count2: 0,
        }
    }

    // -------------------------------------------------------------------------
    // State inspection helpers
    // -------------------------------------------------------------------------

    /// Returns `true` when no count or operator is pending.
    pub fn is_idle(&self) -> bool {
        self.sub_state == SubState::Idle && self.count1 == 0
    }

    /// Short string of the keys accumulated so far, for display in the status bar.
    /// Returns an empty string when the state is clean.
    pub fn pending_keys(&self) -> String {
        let mut s = String::new();
        if self.count1 > 0 {
            s.push_str(&self.count1.to_string());
        }
        if let Some(op) = self.operator {
            s.push(match op {
                Operator::Delete => 'd',
                Operator::Yank => 'y',
            });
        }
        if self.count2 > 0 {
            s.push_str(&self.count2.to_string());
        }
        s
    }

    // -------------------------------------------------------------------------
    // Reset
    // -------------------------------------------------------------------------

    /// Reset to the clean idle state (no pending counts, operator, or sub-state).
    pub fn reset(&mut self) {
        self.sub_state = SubState::Idle;
        self.count1 = 0;
        self.operator = None;
        self.count2 = 0;
    }

    // -------------------------------------------------------------------------
    // Count helpers
    // -------------------------------------------------------------------------

    fn c1(&self) -> usize {
        if self.count1 == 0 {
            1
        } else {
            self.count1
        }
    }

    fn c2(&self) -> usize {
        if self.count2 == 0 {
            1
        } else {
            self.count2
        }
    }

    /// Effective total count for the motion: count1 × count2, each defaulting to 1.
    fn total(&self) -> usize {
        self.c1() * self.c2()
    }

    // -------------------------------------------------------------------------
    // Public entry point
    // -------------------------------------------------------------------------

    /// Feed one character key into the state machine.
    ///
    /// Returns `Some(NormalCommand)` when a complete command has been assembled,
    /// or `None` when more input is expected.
    pub fn process_char(&mut self, ch: char) -> Option<NormalCommand> {
        match self.sub_state {
            SubState::Idle => self.process_idle(ch),
            SubState::OperatorPending => self.process_pending(ch),
        }
    }

    // -------------------------------------------------------------------------
    // Idle state
    // -------------------------------------------------------------------------

    fn process_idle(&mut self, ch: char) -> Option<NormalCommand> {
        match ch {
            // ── Count accumulation ────────────────────────────────────────────
            '1'..='9' => {
                self.count1 = self.count1 * 10 + (ch as usize - '0' as usize);
                None
            }
            '0' => {
                if self.count1 > 0 {
                    // Part of a multi-digit count (e.g. "10j")
                    self.count1 *= 10;
                    None
                } else {
                    // No preceding count → start-of-row motion
                    self.reset();
                    Some(NormalCommand::Move {
                        motion: Motion::StartOfRow,
                        count: 1,
                    })
                }
            }

            // ── Direct navigation motions ─────────────────────────────────────
            'h' => {
                let c = self.c1();
                self.reset();
                Some(NormalCommand::Move {
                    motion: Motion::Left,
                    count: c,
                })
            }
            'j' => {
                let c = self.c1();
                self.reset();
                Some(NormalCommand::Move {
                    motion: Motion::Down,
                    count: c,
                })
            }
            'k' => {
                let c = self.c1();
                self.reset();
                Some(NormalCommand::Move {
                    motion: Motion::Up,
                    count: c,
                })
            }
            'l' => {
                let c = self.c1();
                self.reset();
                Some(NormalCommand::Move {
                    motion: Motion::Right,
                    count: c,
                })
            }
            '$' => {
                self.reset();
                Some(NormalCommand::Move {
                    motion: Motion::EndOfRow,
                    count: 1,
                })
            }

            // ── Operator entry ────────────────────────────────────────────────
            'd' => {
                self.operator = Some(Operator::Delete);
                self.sub_state = SubState::OperatorPending;
                None
            }
            'y' => {
                self.operator = Some(Operator::Yank);
                self.sub_state = SubState::OperatorPending;
                None
            }

            // ── Paste ─────────────────────────────────────────────────────────
            'p' => {
                let c = self.c1();
                self.reset();
                Some(NormalCommand::Paste {
                    before: false,
                    count: c,
                })
            }
            'P' => {
                let c = self.c1();
                self.reset();
                Some(NormalCommand::Paste {
                    before: true,
                    count: c,
                })
            }

            // ── Mode transitions ──────────────────────────────────────────────
            'a' => {
                self.reset();
                Some(NormalCommand::EnterVisualMode)
            }
            'i' => {
                self.reset();
                Some(NormalCommand::EnterInsertMode(Some(String::new())))
            }
            '=' => {
                self.reset();
                Some(NormalCommand::EnterInsertMode(Some("=".to_string())))
            }
            's' | 'S' => {
                self.reset();
                Some(NormalCommand::Save)
            }
            'q' | 'Q' => {
                self.reset();
                Some(NormalCommand::Quit)
            }

            // ── Unknown: reset silently ───────────────────────────────────────
            _ => {
                self.reset();
                None
            }
        }
    }

    // -------------------------------------------------------------------------
    // Operator-pending state
    // -------------------------------------------------------------------------

    fn process_pending(&mut self, ch: char) -> Option<NormalCommand> {
        let op = self
            .operator
            .expect("operator must be set in OperatorPending");

        match ch {
            // ── Count2 accumulation ───────────────────────────────────────────
            '1'..='9' => {
                self.count2 = self.count2 * 10 + (ch as usize - '0' as usize);
                None
            }
            '0' => {
                if self.count2 > 0 {
                    self.count2 *= 10;
                    None
                } else {
                    // '0' as motion → start-of-row
                    let count = self.total();
                    let cmd = Self::op_cmd(op, Motion::StartOfRow, count);
                    self.reset();
                    Some(cmd)
                }
            }

            // ── Motions ───────────────────────────────────────────────────────
            'h' => {
                let c = self.total();
                self.reset();
                Some(Self::op_cmd(op, Motion::Left, c))
            }
            'j' => {
                let c = self.total();
                self.reset();
                Some(Self::op_cmd(op, Motion::Down, c))
            }
            'k' => {
                let c = self.total();
                self.reset();
                Some(Self::op_cmd(op, Motion::Up, c))
            }
            'l' => {
                let c = self.total();
                self.reset();
                Some(Self::op_cmd(op, Motion::Right, c))
            }
            'w' => {
                let c = self.total();
                self.reset();
                Some(Self::op_cmd(op, Motion::WordForward, c))
            }
            'b' => {
                let c = self.total();
                self.reset();
                Some(Self::op_cmd(op, Motion::WordBack, c))
            }
            '$' => {
                let c = self.total();
                self.reset();
                Some(Self::op_cmd(op, Motion::EndOfRow, c))
            }

            // ── Line-wise: dd / yy ────────────────────────────────────────────
            'd' if op == Operator::Delete => {
                let c = self.c1();
                self.reset();
                Some(NormalCommand::DeleteRow { count: c })
            }
            'y' if op == Operator::Yank => {
                let c = self.c1();
                self.reset();
                Some(NormalCommand::YankRow { count: c })
            }

            // ── Anything else: cancel ─────────────────────────────────────────
            _ => {
                self.reset();
                Some(NormalCommand::Reset)
            }
        }
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    fn op_cmd(op: Operator, motion: Motion, count: usize) -> NormalCommand {
        match op {
            Operator::Delete => NormalCommand::Delete { motion, count },
            Operator::Yank => NormalCommand::Yank { motion, count },
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_hjkl() {
        let mut s = NormalModeState::new();
        assert!(matches!(
            s.process_char('h'),
            Some(NormalCommand::Move {
                motion: Motion::Left,
                count: 1
            })
        ));
        assert!(matches!(
            s.process_char('j'),
            Some(NormalCommand::Move {
                motion: Motion::Down,
                count: 1
            })
        ));
        assert!(matches!(
            s.process_char('k'),
            Some(NormalCommand::Move {
                motion: Motion::Up,
                count: 1
            })
        ));
        assert!(matches!(
            s.process_char('l'),
            Some(NormalCommand::Move {
                motion: Motion::Right,
                count: 1
            })
        ));
    }

    #[test]
    fn test_count_navigation() {
        let mut s = NormalModeState::new();
        assert!(s.process_char('5').is_none());
        let cmd = s.process_char('j');
        assert!(matches!(
            cmd,
            Some(NormalCommand::Move {
                motion: Motion::Down,
                count: 5
            })
        ));
    }

    #[test]
    fn test_dd() {
        let mut s = NormalModeState::new();
        assert!(s.process_char('d').is_none());
        assert!(matches!(
            s.process_char('d'),
            Some(NormalCommand::DeleteRow { count: 1 })
        ));
    }

    #[test]
    fn test_2dd() {
        let mut s = NormalModeState::new();
        s.process_char('2');
        s.process_char('d');
        assert!(matches!(
            s.process_char('d'),
            Some(NormalCommand::DeleteRow { count: 2 })
        ));
    }

    #[test]
    fn test_dj() {
        let mut s = NormalModeState::new();
        s.process_char('d');
        assert!(matches!(
            s.process_char('j'),
            Some(NormalCommand::Delete {
                motion: Motion::Down,
                count: 1
            })
        ));
    }

    #[test]
    fn test_2d2j() {
        let mut s = NormalModeState::new();
        s.process_char('2');
        s.process_char('d');
        s.process_char('2');
        let cmd = s.process_char('j');
        assert!(matches!(
            cmd,
            Some(NormalCommand::Delete {
                motion: Motion::Down,
                count: 4
            })
        ));
    }

    #[test]
    fn test_yy() {
        let mut s = NormalModeState::new();
        s.process_char('y');
        assert!(matches!(
            s.process_char('y'),
            Some(NormalCommand::YankRow { count: 1 })
        ));
    }

    #[test]
    fn test_paste() {
        let mut s = NormalModeState::new();
        assert!(matches!(
            s.process_char('p'),
            Some(NormalCommand::Paste {
                before: false,
                count: 1
            })
        ));
        assert!(matches!(
            s.process_char('P'),
            Some(NormalCommand::Paste {
                before: true,
                count: 1
            })
        ));
    }

    #[test]
    fn test_operator_cancel_on_esc_equivalent() {
        let mut s = NormalModeState::new();
        s.process_char('d');
        // Unknown char cancels
        assert!(matches!(s.process_char('x'), Some(NormalCommand::Reset)));
        assert!(s.is_idle());
    }

    #[test]
    fn test_pending_keys_display() {
        let mut s = NormalModeState::new();
        s.process_char('2');
        assert_eq!(s.pending_keys(), "2");
        s.process_char('d');
        assert_eq!(s.pending_keys(), "2d");
        s.process_char('3');
        assert_eq!(s.pending_keys(), "2d3");
    }

    #[test]
    fn test_zero_as_motion() {
        let mut s = NormalModeState::new();
        // '0' with no count → start of row
        assert!(matches!(
            s.process_char('0'),
            Some(NormalCommand::Move {
                motion: Motion::StartOfRow,
                ..
            })
        ));
    }

    #[test]
    fn test_zero_as_count_part() {
        let mut s = NormalModeState::new();
        s.process_char('1');
        s.process_char('0'); // count1 becomes 10
        let cmd = s.process_char('j');
        assert!(matches!(
            cmd,
            Some(NormalCommand::Move {
                motion: Motion::Down,
                count: 10
            })
        ));
    }
}
