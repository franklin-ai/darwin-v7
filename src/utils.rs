#[macro_export]
macro_rules! expect_http_ok {
    ($x: ident, $y: ty) => {
        if $x.status() != 200 {
            bail!(format!(
                "Invalid status code {} {}",
                $x.status(),
                $x.text().await?
            ))
        } else {
            let text = $x.text().await?;
            let res = serde_json::from_str::<$y>(&text);
            if (res.is_err()) {
                bail!("{}\n{}", text, res.err().unwrap())
            }

            Ok(res?)
        }
    };
}
