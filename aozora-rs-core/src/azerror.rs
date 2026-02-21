/// 非致命的エラーを蓄積する型、AZResultのコンストラクタです。
pub struct AZResultC {
    errors: Vec<miette::Error>,
}

impl AZResultC {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// エラーを内部に蓄積します。
    pub fn push(&mut self, e: miette::Error) {
        self.errors.push(e);
    }

    /// 蓄積したエラーで値を包み、AZResult<T>へと型を確定させます。
    pub fn finally<T>(self, result: T) -> AZResult<T> {
        AZResult {
            inside: result,
            errors: self.errors,
        }
    }
}

impl From<Vec<miette::Error>> for AZResultC {
    fn from(value: Vec<miette::Error>) -> Self {
        Self { errors: value }
    }
}

/// Graceful Degradationに対応するためのResult型です。非致命的エラーをerrorsの中に蓄積します。
///
/// 自分でこの型を構築したい場合、コンストラクタ型であるAZResultCを利用してください。
pub struct AZResult<T> {
    inside: T,
    errors: Vec<miette::Error>,
}

impl<T> AZResult<T> {
    /// 標準エラー出力にエラーを出力して中の値を所有権付きで取得します。
    pub fn unpack(self) -> T {
        for e in self.errors {
            eprintln!("{:?}", e);
        }
        self.inside
    }

    /// 中の値とエラーをタプルとして所有権付きで取得します。
    pub fn into_tuple(self) -> (T, Vec<miette::Error>) {
        (self.inside, self.errors)
    }
}
