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
            Ok(serde_path_to_error::deserialize(
                &mut serde_json::Deserializer::from_str(&text),
            )?)
        }
    };
}
