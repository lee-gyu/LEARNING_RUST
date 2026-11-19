//! 가계부 본체와 감사 기록

use std::sync::Mutex;

use crate::entry::{Category, Entry};
use crate::report::{self, Style};

/// 항목 번호를 1부터 차례로 만듭니다.
#[derive(Default)]
pub struct IdGen {
    last: u32,
}

impl IdGen {
    pub fn gen(&mut self) -> u32 {
        self.last += 1;
        self.last
    }
}

/// 무엇이 언제 시작하고 끝났는지 남기는 감사 기록
#[derive(Default)]
pub struct Audit {
    events: Mutex<Vec<String>>,
}

impl Audit {
    pub fn note(&self, event: &str) {
        self.events.lock().unwrap().push(event.to_string());
    }

    /// 구간을 시작합니다. 돌려받은 `Scope`가 버려질 때 구간이 끝납니다.
    pub fn enter(&self, name: &'static str) -> Scope<'_> {
        self.note(&format!("{} 시작", name));
        Scope { audit: self, name }
    }

    pub fn events(&self) -> Vec<String> {
        self.events.lock().unwrap().clone()
    }
}

pub struct Scope<'a> {
    audit: &'a Audit,
    name: &'static str,
}

impl Scope<'_> {
    /// 구간 안에서 `f`를 실행하고 그 결과를 돌려줍니다.
    pub fn run<T, F: FnOnce() -> T>(&self, f: F) -> T {
        f()
    }
}

impl Drop for Scope<'_> {
    fn drop(&mut self) {
        self.audit.note(&format!("{} 끝", self.name));
    }
}

/// 정산 작업: 제목과, 작업이 끝날 때 닫히는 감사 구간
struct Job<'a> {
    title: String,
    _scope: Scope<'a>,
}

#[derive(Default)]
pub struct Ledger {
    entries: Vec<Entry>,
    ids: IdGen,
    /// 카테고리별 합계 캐시(`Category as usize` 순서). `&self` 메서드에서 채우므로 `Mutex`에 넣었습니다.
    totals: Mutex<[Option<i64>; 4]>,
    pub audit: Audit,
}

impl Ledger {
    pub fn new() -> Ledger {
        Ledger::default()
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// 항목을 더하고 새 번호를 돌려줍니다.
    pub fn add(&mut self, entry: Entry) -> u32 {
        self.entries.push(entry);
        *self.totals.lock().unwrap() = [None; 4];
        self.ids.gen()
    }

    pub fn add_line(&mut self, line: &str) -> Result<u32, String> {
        let entry = Entry::parse(line)?;
        Ok(self.add(entry))
    }

    /// 줄 하나를 더합니다. 잘못된 줄이면 패닉합니다.
    // 2018 에디션에서 인자가 하나뿐인 panic!은 메시지를 서식 처리하지 않습니다(경고를 막아 둠).
    #[allow(non_fmt_panics)]
    pub fn push_line(&mut self, line: &str) -> u32 {
        match self.add_line(line) {
            Ok(id) => id,
            Err(_) => panic!("잘못된 줄: {line}"),
        }
    }

    /// 지금까지 붙인 번호들
    pub fn ids(&self) -> impl Iterator<Item = u32> {
        1..=self.ids.last
    }

    /// 카테고리별 합계. 한 번 계산한 값은 캐시에 둡니다.
    pub fn total_of(&self, category: Category) -> i64 {
        let slot = category as usize;
        let total = if let Some(cached) = self.totals.lock().unwrap()[slot] {
            cached
        } else {
            self.entries
                .iter()
                .filter(|e| e.category == category)
                .map(|e| e.amount)
                .sum()
        };
        self.totals.lock().unwrap()[slot] = Some(total);
        total
    }

    /// 총액. 감사 기록에 "잔액" 구간과 그 안의 "합산" 구간을 남깁니다.
    pub fn balance(&self) -> i64 {
        let _scope = self.audit.enter("잔액");
        self.audit
            .enter("합산")
            .run(|| self.entries.iter().map(|e| e.amount).sum())
    }

    /// 월말 정산 보고서. 만드는 동안 감사 기록에 "정산" 구간을 남깁니다.
    #[allow(bare_trait_objects)]
    pub fn close_month(&self, style: &Style) -> String {
        let job = Job {
            title: format!("{}건 정산", self.entries.len()),
            _scope: self.audit.enter("정산"),
        };
        let header = move || format!("== {} ==", job.title);
        let body = report::render(self, style);
        self.audit.note("보고서 완성");
        format!("{}\n{}", header(), body)
    }
}
