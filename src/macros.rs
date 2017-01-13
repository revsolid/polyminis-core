#[macro_export]
macro_rules! get_value_or_do 
{
    ($e:expr, $block:expr) =>
    (
        match $e
        {
            Some(val) => val,
            None => { $block }
        }
    );
}
