#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


use ::futures::{executor::LocalPool, task::LocalSpawnExt, FutureExt};
use nwd::NwgUi;

use ::nwg::{self as nwg, GridLayout, Icon, Monitor, NativeUi, Window};
use ::nwg_webview_ctrl::{WebviewContainer, WebviewContainerFlags};
use std::cell::RefCell;
use ::std::error::Error;


#[derive(Default, NwgUi)]
pub struct DemoUi {
    #[nwg_resource(source_bin: Some(include_bytes!("../assets/weibo.ico")), size: Some((16, 16)), strict: true)]
    app_icon: Icon,

    #[nwg_control(size: DemoUi::SIZE, position: DemoUi::position(), icon: Some(&data.app_icon), title: "内嵌 WebView 例程", flags: "MAIN_WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [nwg::stop_thread_dispatch()])]
    window: Window,

    #[nwg_layout(margin: [0; 4], parent: window, max_row: Some(1), max_column: Some(2), spacing: 0)]
    grid: GridLayout,

    // #[nwg_layout(margin: [0; 4], parent: window, max_row: Some(1), max_column: Some(2), spacing: 0)]
    // grid1: GridLayout,

    #[nwg_control(flags: "VISIBLE", parent: window, window: &data.window, language: "en_us")]
    #[nwg_layout_item(layout: grid, row: 0, col: 1)]
    webview_container: WebviewContainer,

    #[nwg_layout(margin: [0; 4], parent: window, max_row: Some(6), max_column: Some(1), spacing: 0)]
    // #[nwg_layout_item(layout: grid, row: 0, col: 0)]
    grid2: GridLayout,

    #[nwg_control(text: "Say my name")]
    #[nwg_layout_item(layout: grid2, row: 0, col: 0)]
    #[nwg_events( OnButtonClick: [DemoUi::say_hello] )]
    hello_button: nwg::Button,

    #[nwg_control(text: "addItem")]
    #[nwg_layout_item(layout: grid2, row: 1, col: 0)]
    #[nwg_events( OnButtonClick: [DemoUi::add_item] )]
    hello_button2: nwg::Button,

    buttons: RefCell<Vec<nwg::Button>>,
    handlers: RefCell<Vec<nwg::EventHandler>>,
}

impl DemoUi {
    const SIZE: (i32, i32) = (1024, 768);
    fn say_hello(&self) {
        let (_env, _contrller, webview) = self.webview_container.ready_block().unwrap();
        let _ = webview.execute_script("document.cookie", move |js_cookie| {
            nwg::simple_message("title", &js_cookie);
            fetch(js_cookie);
            Ok(())
        });
    }
    fn add_item(&self) {
        let mut new_button = Default::default();
        nwg::Button::builder()
            .text("11")
            .size((20, 10))
            .parent(&self.window)
            .build(&mut new_button)
            .expect("Failed to build button");

        let mut buttons = self.buttons.borrow_mut();
        let mut handlers = self.handlers.borrow_mut();

        let blen = buttons.len() as u32;
        let x = blen + 2;

        if x < 6 {
            self.grid2.add_child(0, x, &new_button);
            self.grid.min_size([10, 10]);

            let new_button_handle = new_button.handle;
            let handler = nwg::bind_event_handler(
                &new_button.handle,
                &self.window.handle,
                move |evt, _evt_data, handle| match evt {
                    nwg::Event::OnButtonClick => {
                        if handle == new_button_handle {
                            nwg::simple_message("title", "&content");
                        }
                    }
                    _ => {}
                },
            );

            buttons.push(new_button);
            handlers.push(handler);
        } else {
            // self.grid.remove_child_by_pos(1, 0);
            self.window.set_visible(false);
        }
    }

    /// 主窗体初始显示位置
    fn position() -> (i32, i32) {
        (
            (Monitor::width() - Self::SIZE.0) / 2,
            (Monitor::height() - Self::SIZE.1) / 2,
        )
    }
    /// 业务处理逻辑封装成员方法
    pub fn executor(&self, url: &'static str) -> Result<LocalPool, Box<dyn Error>> {
        let executor = LocalPool::new();
        let webview_ready_fut = self.webview_container.ready_fut()?;
        executor.spawner().spawn_local(
            async move {
                let (_, _, webview) = webview_ready_fut.await;
                webview.navigate(url)?;
                Ok::<_, Box<dyn Error>>(())
            }
            .map(|result| {
                if let Err(err) = result {
                    eprintln!("[app_main]{err}");
                }
            }),
        )?;
        Ok(executor)
    }
}

fn fetch(cookie: String) {
    nwg::simple_message("title", &cookie);
    // let file = std::fs::File::create("weibo_cookie.json");
    std::fs::write("cookie.json", cookie.as_bytes());
}

fn main() -> Result<(), Box<dyn Error>> {

    nwg::init()?;
    // 主窗体
    let demo_ui_app = DemoUi::build_ui(Default::default())?;
    // 业务处理逻辑
    let mut executor = demo_ui_app.executor("https://weibo.com")?;
    // 阻塞主线程，等待用户手动关闭主窗体
    nwg::dispatch_thread_events_with_callback(move ||
        // 以 win32 UI 的事件循环为【反应器】，对接 futures crate 的【执行器】
        executor.run_until_stalled());
    Ok(())
}


