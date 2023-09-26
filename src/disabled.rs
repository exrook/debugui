#[macro_export]
macro_rules! set {
    ($name:literal, $value:expr => $([$egui:ident])? |_ : & $data_ty:ty, $ui_data:pat_param, $ui:ident| $fn_body:block ) => {{
        $value
    }};
    ($name:literal, $value:expr => $([$egui:ident])? |$data:ident : & $data_ty:ty, $ui_data:pat_param, $ui:ident| $fn_body:block ) => {{
        $value
    }};
    ($value:expr => $([$egui:ident])? | _ : & $data_ty:ty, $ui_data:pat_param, $ui:ident| $fn_body:block ) => {{
        $value
    }};
    ($value:expr => $([$egui:ident])? |$data:ident : & $data_ty:ty, $ui_data:pat_param, $ui:ident| $fn_body:block ) => {{
        $value
    }};
    ($name:literal, $value:expr $( => $viewer:ident)?) => {{
        $value
    }};
    ($value:expr $(=> $viewer:ident)?) => {{
        $value
    }};
}

#[macro_export]
macro_rules! viewer {
    ($($viewer:tt)*) => {};
}
#[macro_export]
macro_rules! init_on {
    ($resources:ident, $loop:expr, $instance:expr, $adapter:expr, $device:expr, $queue:expr) => {};
}

#[macro_export]
macro_rules! feed_on {
    ($resources:ident, $event:expr, $control_flow:expr) => {
        false
    };
}
