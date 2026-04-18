use std::fmt::Debug;

/// 非致命的エラーを蓄積する型、AZResultのコンストラクタです。
#[derive(Default)]
pub struct AZResultC<E>
where
    E: Default + Debug,
{
    errors: Vec<E>,
}

impl<E> AZResultC<E>
where
    E: Default + Debug,
{
    /// エラーを内部に蓄積します。
    pub fn acc_err(&mut self, e: E) {
        self.errors.push(e);
    }

    /// 蓄積したエラーで値を包み、AZResult<T>へと型を確定させます。
    pub fn finally<T>(self, result: T) -> AZResult<T, E> {
        AZResult {
            inside: result,
            errors: self.errors,
        }
    }
}

impl<E> From<Vec<E>> for AZResultC<E>
where
    E: Default + Debug,
{
    fn from(value: Vec<E>) -> Self {
        Self { errors: value }
    }
}

/// Graceful Degradationに対応するためのResult型です。非致命的エラーをerrorsの中に蓄積します。
///
/// 自分でこの型を構築したい場合、コンストラクタ型であるAZResultCを利用してください。
pub struct AZResult<T, E>
where
    E: Debug,
{
    inside: T,
    errors: Vec<E>,
}

impl<T, E> AZResult<T, E>
where
    E: Debug,
{
    /// 標準エラー出力にエラーを出力して中の値を所有権付きで取得します。
    pub fn unpack(self) -> T {
        for e in self.errors {
            eprintln!("{:?}", e);
        }
        self.inside
    }

    /// 中の値とエラーをタプルとして所有権付きで取得します。
    pub fn into_tuple(self) -> (T, Vec<E>) {
        (self.inside, self.errors)
    }
}
