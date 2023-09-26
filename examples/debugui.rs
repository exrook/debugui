debugui::viewer! {
    [egui]
    fn draw(original: &MyType, ui_value: &mut Option<MyType>, ui) {
        let mut b = ui_value.is_some();
            ui.label(format!("Progam is setting values: {} {}", original.value, original.value2));
        ui.checkbox(&mut b, "Enable override");
        if b {
            let my_setting = ui_value.get_or_insert_with(||original.clone());
            ui.add(egui::widgets::Slider::new(&mut my_setting.value, 0..=256));
            ui.add(egui::widgets::Slider::new(&mut my_setting.value2, -200.0..=200.0));
        } else {
            *ui_value = None;
        }
    }
}

#[derive(Clone, Debug)]
struct MyType {
    value: u32,
    value2: f32,
}
impl debugui::AsDebuggableParam for MyType {
    type Value = Self;
    fn get_value(&self) -> &Self {
        self
    }
    fn set_value(&mut self, new_value: &Self) {
        self.clone_from(new_value)
    }
}

fn main() {
    let mut mine = MyType {
        value: 62,
        value2: 6.28,
    };
    let mut mine2 = MyType {
        value: 62,
        value2: 6.28,
    };
    let param = debugui::set!(35);
    loop {
        let param = debugui::set!("NAMED", 35 => |_: &u32, o, ui| {
            ui.label(format!("VALUE: {:?}", o));
        });
        let mine = debugui::set!(&mut mine);
        let mine2 = debugui::set!(&mut mine2 => |original: &MyType, ui_value, ui| {
            ui.label(format!("Progam is setting AAA values: {} {}", original.value, original.value2));
        });
        println!("Parameter is {:?}", param);
        println!("Mine is {:?} AND {:?}", mine, mine2);
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}
