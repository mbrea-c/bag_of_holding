#[macro_export]
macro_rules! simulated_server_auth {
    () => { Or<(With<lightyear::prelude::Replicating>, With<lightyear::prelude::client::Predicted>)> };
}
