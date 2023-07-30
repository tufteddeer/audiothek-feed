#[macro_export]
macro_rules! add_template_file {
    ($tera:ident, $dir:expr, $file:tt) => {
        $tera.add_raw_template($file, include_str!(concat!($dir, concat!("/", $file))))
    };
}
