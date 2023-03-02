#[macro_export]
macro_rules! expect_http_ok {
    ($x: ident, $y: ty) => {
        if $x.status() != 200 {
            bail!(format!("Invalid status code {}", $x.status()))
        } else {
            Ok($x.json::<$y>().await?)
        }
    };
}
