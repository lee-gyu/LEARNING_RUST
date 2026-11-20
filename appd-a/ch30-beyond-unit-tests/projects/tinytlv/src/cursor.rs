//! 입력 바이트열을 앞에서부터 읽는 커서입니다(비공개 모듈).

/// 읽은 위치를 기억하며 바이트를 꺼냅니다.
///
/// 약속: 어떤 인자에도 패닉하지 않습니다. 모자라면 위치를 옮기지 않고 `None`을 돌려줍니다.
pub(crate) struct Cursor<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    /// 다음에 읽을 위치입니다.
    pub(crate) fn pos(&self) -> usize {
        self.pos
    }

    /// 아직 읽지 않은 바이트 수입니다.
    pub(crate) fn remaining(&self) -> usize {
        self.input.len() - self.pos
    }

    /// 다음 1바이트를 꺼냅니다.
    pub(crate) fn byte(&mut self) -> Option<u8> {
        let b = *self.input.get(self.pos)?;
        self.pos += 1;
        Some(b)
    }

    /// 다음 `n`바이트를 꺼냅니다.
    pub(crate) fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        let out = self.input.get(self.pos..end)?;
        self.pos = end;
        Some(out)
    }
}

// 단위 테스트: 비공개 타입 `Cursor`를 직접 부릅니다. 통합 테스트에서는 이 타입이 보이지 않습니다.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_up_to_the_exact_end() {
        let mut c = Cursor::new(&[1, 2, 3]);
        assert_eq!(c.take(3), Some(&[1, 2, 3][..]));
        assert_eq!(c.remaining(), 0);
        assert_eq!(c.byte(), None);
    }

    #[test]
    fn short_take_keeps_position() {
        let mut c = Cursor::new(&[1, 2, 3]);
        assert_eq!(c.byte(), Some(1));
        assert_eq!(c.take(3), None);
        // 실패해도 위치가 그대로여야 parse가 에러 위치를 정확히 알릴 수 있습니다.
        assert_eq!(c.pos(), 1);
        assert_eq!(c.take(2), Some(&[2, 3][..]));
    }

    #[test]
    fn take_zero_is_an_empty_slice() {
        let mut c = Cursor::new(&[]);
        assert_eq!(c.take(0), Some(&[][..]));
    }

    /// `parse`는 길이를 1바이트에서 읽으므로 `n`이 255를 넘는 일이 없습니다.
    /// 그래서 이 경로(`pos + n`의 넘침)는 공개 API로는 닿을 수 없고, 단위 테스트만 확인할 수 있습니다.
    #[test]
    fn huge_take_does_not_overflow() {
        let mut c = Cursor::new(&[1, 2]);
        assert_eq!(c.byte(), Some(1));
        assert_eq!(c.take(usize::MAX), None);
        assert_eq!(c.pos(), 1);
    }
}
