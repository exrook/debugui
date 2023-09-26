#[macro_export]
macro_rules! set {
    ($name:literal, $value:expr => $([$egui:ident])? |_ : & $data_ty:ty, $ui_data:pat_param, $ui:ident| $fn_body:block ) => {{
        let tmp = $value;
        tmp
    }};
    ($name:literal, $value:expr => $([$egui:ident])? |$data:ident : & $data_ty:ty, $ui_data:pat_param, $ui:ident| $fn_body:block ) => {{
        let tmp = $value;
        tmp
    }};
    ($value:expr => $([$egui:ident])? | _ : & $data_ty:ty, $ui_data:pat_param, $ui:ident| $fn_body:block ) => {{
        let tmp = $value;
        tmp
    }};
    ($value:expr => $([$egui:ident])? |$data:ident : & $data_ty:ty, $ui_data:pat_param, $ui:ident| $fn_body:block ) => {{
        let tmp = $value;
        tmp
    }};
    ($name:literal, $value:expr $( => $viewer:ident)?) => {{
        let tmp = $value;
        tmp
    }};
    ($value:expr $(=> $viewer:ident)?) => {{
        let tmp = $value;
        tmp
    }};
}

#[macro_export]
macro_rules! viewer {
    ($($viewer:tt)*) => {};
}
#[macro_export]
macro_rules! init_on {
    ($resources:ident, $loop:expr, $instance:expr, $adapter:expr, $device:expr, $queue:expr) => {
        let $resources = ($device, $queue);
    };
}

#[macro_export]
macro_rules! feed_on {
    ($resources:ident, $event:expr, $control_flow:expr) => {{
        // avoid compiler warning of unused variable $resources when disabled
        let _used = $resources;
        false
    }};
}
