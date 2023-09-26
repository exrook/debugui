use parking_lot::RwLock;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    sync::OnceLock,
};

pub use egui;
pub use linkme;

use super::AsDebuggableParam;

pub mod ui;

#[macro_export]
macro_rules! set {
    ($name:literal, $value:expr => $([$egui:ident])? |_ : & $data_ty:ty, $ui_data:pat_param, $ui:ident| $fn_body:block ) => {{
        $crate::set!(@internal_fn $name, $value => $([$egui])? |[_] : & $data_ty, $ui_data, $ui| $fn_body)
    }};
    ($name:literal, $value:expr => $([$egui:ident])? |$data:ident : & $data_ty:ty, $ui_data:pat_param, $ui:ident| $fn_body:block ) => {{
        $crate::set!(@internal_fn $name, $value => $([$egui])? |[$data] : & $data_ty, $ui_data, $ui| $fn_body)
    }};
    ($value:expr => $([$egui:ident])? | _ : & $data_ty:ty, $ui_data:pat_param, $ui:ident| $fn_body:block ) => {{
        const NAME: &str = concat!(file!(), ":", line!());
        $crate::set!(@internal_fn NAME, $value => $([$egui])? |[_] : &$data_ty, $ui_data, $ui| $fn_body)
    }};
    ($value:expr => $([$egui:ident])? |$data:ident : & $data_ty:ty, $ui_data:pat_param, $ui:ident| $fn_body:block ) => {{
        const NAME: &str = concat!(file!(), ":", line!());
        $crate::set!(@internal_fn NAME, $value => $([$egui])? |[$data] : &$data_ty, $ui_data, $ui| $fn_body)
    }};
    (@internal_fn $name:expr, $value:expr => $([$egui:ident])? |[$data:pat] : &$data_ty:ty, $ui_data:pat, $ui:ident| $fn_body:block ) => {{
        pub static VIEWER: &$crate::DynViewer = {
            $(use $crate::egui as $egui;)?
            let viewer = |$data: &$data_ty, $ui_data: &mut Option<$data_ty>, $ui: &mut $crate::egui::Ui| $fn_body;
            &$crate::AViewerFn::new(viewer)
        };

        #[$crate::linkme::distributed_slice($crate::PARAMS)]
        #[linkme(crate = $crate::linkme)]
        static PARAM: $crate::DebugParam = $crate::DebugParam::custom_viewer($name, VIEWER);

        let mut val = $value;
        if false {
            fn types_match( _: &<$data_ty as $crate::AsDebuggableParam>::Value) {}
            types_match(&val);
        }
        PARAM.set(&mut val);
        val
    }};
    ($name:literal, $value:expr $( => $viewer:ident)?) => {{
        const NAME: &str = $name;
        $crate::set!(@internal NAME, $value $(: $viewer)?)
    }};
    ($value:expr $(=> $viewer:ident)?) => {{
        const NAME: &str = concat!(file!(), ":", line!());
        $crate::set!(@internal NAME, $value $(=> $viewer)?)
    }};
    (@internal $name:ident, $value:expr => $viewer:ident) => {{
        #[$crate::linkme::distributed_slice($crate::PARAMS)]
        #[linkme(crate = $crate::linkme)]
        static PARAM: $crate::DebugParam = $crate::DebugParam::custom_viewer($name, $viewer);
        let mut val = $value;
        PARAM.set(&mut val);
        val
    }};
    (@internal $name:ident, $value:expr) => {{
        #[$crate::linkme::distributed_slice($crate::PARAMS)]
        #[linkme(crate = $crate::linkme)]
        static PARAM: $crate::DebugParam = $crate::DebugParam::new($name);
        let mut val = $value;
        PARAM.set(&mut val);
        val
    }};
}

#[macro_export]
macro_rules! viewer {
    ($([$egui:ident])? pub fn $viewer:ident ( $data:ident : & $data_ty:ty, $ui_data:ident : $uidata_ty:ty,  $ui:ident ) $body:block ) => {
        static $viewer : &$crate::DynViewer = $crate::viewer!(@internal $([$egui])? fn $viewer ( $data : & $data_ty, $ui_data : $uidata_ty,  $ui) $body return);
    };
    ($([$egui:ident])? fn $viewer:ident ( $data:ident : & $data_ty:ty, $ui_data:ident : $uidata_ty:ty,  $ui:ident ) $body:block ) => {
        const _: () = {
            $crate::viewer!(@internal $([$egui])? fn $viewer ( $data : & $data_ty, $ui_data : $uidata_ty,  $ui) $body);
        };
    };
    (@internal $([$egui:ident])? fn $viewer:ident ( $data:ident : & $data_ty:ty, $ui_data:ident : $uidata_ty:ty,  $ui:ident ) $body:block $($return:ident)?) => {
        {
            $(use $crate::egui as $egui;)?
            fn $viewer($data : &$data_ty, $ui_data: $uidata_ty , $ui: &mut egui::Ui ) $body
            #[linkme::distributed_slice($crate::VIEWERS)]
            pub static VIEWER: &$crate::DynViewer = &$crate::AViewerFn::new($viewer);
            $($crate::viewer!(@ $return VIEWER) )?
        }
    };
    (@return $ident:ident) => {
        $ident
    };
    (@static $export:ident $ty:ty $block:block) => {
        static $export: $ty = $block;
    }
}

pub trait ViewerFnTrait: Sized {
    fn draw(original: &Self, ui_value: &mut Option<Self>, ui: &mut egui::Ui);
    fn internal() {}
}

#[linkme::distributed_slice]
pub static PARAMS: [DebugParam] = [..];

#[linkme::distributed_slice]
pub static VIEWERS: [&DynViewer] = [..];

static VIEWER_TABLE: OnceLock<HashMap<TypeId, &'static DynViewer>> = OnceLock::new();

fn viewer_for<T: 'static>() -> Option<&'static DynViewer> {
    viewer_for_typeid(TypeId::of::<T>())
}

fn viewer_for_typeid(id: TypeId) -> Option<&'static DynViewer> {
    let table = VIEWER_TABLE.get_or_init(|| {
        VIEWERS
            .iter()
            .filter_map(|v| v.param_type().map(|t| (t, *v)))
            .collect()
    });
    table.get(&id).copied()
}

pub type DynAny = (dyn Any + 'static + Send + Sync);
pub type DynViewer = dyn ViewerInternal + 'static + Send + Sync;

struct NullViewer;

impl ViewerInternal for NullViewer {
    fn draw(&self, _: &DynAny, ui: &mut egui::Ui) {
        ui.label("no viewer found");
    }
    fn is_for(&self, _: &DynAny) -> bool {
        true
    }
}

pub trait ViewerInternal {
    fn param_type(&self) -> Option<TypeId> {
        None
    }
    fn is_for(&self, _param: &DynAny) -> bool {
        false
    }
    fn draw(&self, param: &DynAny, ui: &mut egui::Ui);
}

pub struct AViewerFn<F, T>(F, PhantomData<T>);

impl<F: Fn(&T, &mut Option<T>, &mut egui::Ui), T> AViewerFn<F, T> {
    pub const fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<F: Fn(&T, &mut Option<T>, &mut egui::Ui), T: 'static> ViewerInternal for AViewerFn<F, T> {
    fn param_type(&self) -> Option<TypeId> {
        Some(TypeId::of::<T>())
    }
    fn is_for(&self, param: &DynAny) -> bool {
        param.is::<ParamStorage<T>>()
    }
    fn draw(&self, param: &DynAny, ui: &mut egui::Ui) {
        let storage: &ParamStorage<T> = param.downcast_ref::<ParamStorage<T>>().unwrap();
        let mut inner = storage.inner.write();
        let (prog_val, ui_val) = inner.split_values();
        self.0(prog_val, ui_val, ui);
    }
}

pub struct DebugParam {
    name: &'static str,
    inner: OnceLock<(Box<DynAny>, &'static DynViewer)>,
    custom_viewer: Option<&'static DynViewer>,
}

impl DebugParam {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            inner: OnceLock::new(),
            custom_viewer: None,
        }
    }
    pub const fn custom_viewer(name: &'static str, viewer: &'static DynViewer) -> Self {
        Self {
            name,
            inner: OnceLock::new(),
            custom_viewer: Some(viewer),
        }
    }
    pub fn set<P: AsDebuggableParam>(&'static self, param: &mut P) {
        ui::try_launch_own_thread();
        let (storage_box, _) = self.inner.get_or_init(|| {
            (
                Box::new(ParamStorage::<P::Value>::new(&*param)),
                self.custom_viewer
                    .or_else(viewer_for::<P::Value>)
                    .unwrap_or(&NullViewer),
            )
        });
        // dereference is required to so we downcast the &dyn instead of the &Box<dyn _>
        let storage: &ParamStorage<P::Value> = (*storage_box).downcast_ref().unwrap();
        storage.set(param);
    }
}

struct ParamStorage<T> {
    inner: RwLock<ParamInner<T>>,
}

struct ParamInner<T> {
    program_val: T,
    ui_val: Option<T>,
}

impl<T> ParamInner<T> {
    fn split_values(&mut self) -> (&T, &mut Option<T>) {
        (&self.program_val, &mut self.ui_val)
    }
}

impl<T: 'static + Send + Sync + Clone> ParamStorage<T> {
    fn set<P: AsDebuggableParam<Value = T>>(&self, param: &mut P) {
        let mut inner = self.inner.write();
        inner.program_val.clone_from(param.get_value());

        if let Some(ref ui_val) = inner.ui_val {
            param.set_value(ui_val);
        }
    }

    fn new<P: AsDebuggableParam<Value = T>>(param: &P) -> Self {
        Self {
            inner: RwLock::new(ParamInner {
                program_val: param.get_value().clone(),
                ui_val: None,
            }),
        }
    }
}

macro_rules! numeric_viewer {
    ($($t:ty),+ $(,)?) => {
        $(numeric_viewer!(@ $t);)*
    };
    (@ $t:ty) => {
        $crate::viewer! {
            fn draw(program: &$t, ui_value: &mut Option<$t>, ui) {
                ui.label(format!("Progam provided value: {}", program));

                let mut b = ui_value.is_some();
                ui.checkbox(&mut b, "Override");
                if b {
                    let ui_setting = ui_value.get_or_insert_with(||program.clone());
                    ui.add(egui::widgets::DragValue::new(ui_setting));
                } else {
                    *ui_value = None;
                }
            }
        }
    };
}

numeric_viewer!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
