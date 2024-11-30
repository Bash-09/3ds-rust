use ctru::prelude::*;

pub trait App {
    fn init(&mut self, ctx: &mut Context);
    fn update(&mut self, ctx: &mut Context) -> MainLoopAction;
    fn render(&mut self, ctx: &mut Context);
}

pub struct Context<'a> {
    pub apt: Apt,
    pub hid: Hid,
    pub gfx: &'a Gfx,
}

#[derive(Default)]
pub enum MainLoopAction {
    #[default]
    Nothing,
    Exit,
}

pub fn run<A: App>(mut app: A, ctx: &mut Context) {
    app.init(ctx);

    while ctx.apt.main_loop() {
        ctx.hid.scan_input();

        if let MainLoopAction::Exit = app.update(ctx) {
            break;
        }

        app.render(ctx);

        ctx.gfx.wait_for_vblank();
    }
}
