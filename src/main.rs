#![feature(seek_stream_len)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ::futures::{executor::LocalPool, task::LocalSpawnExt, FutureExt};
use nwd::NwgUi;

use ::nwg::{self as nwg, GridLayout, Icon, Monitor, NativeUi, Window};
use ::nwg_webview_ctrl::{WebviewContainer, WebviewContainerFlags};
use ::std::error::Error;
use std::cell::{ RefCell};
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Seek, Write};
use std::path::Path;
use nwg::{ Frame};


#[derive(Default, NwgUi)]
pub struct DemoUi {
    #[nwg_resource(source_bin: Some(include_bytes!("../assets/weibo.ico")), size: Some((16, 16)), strict: true)]
    app_icon: Icon,

    #[nwg_control(size: DemoUi::SIZE, position: DemoUi::position(), icon: Some(&data.app_icon), title: "内嵌 WebView 例程", flags: "MAIN_WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [nwg::stop_thread_dispatch()])]
    window: Window,

    #[nwg_layout(margin: [0; 4], parent: window, max_row: Some(1), max_column: Some(2), spacing: 0)]
    grid: GridLayout,

    #[nwg_control(flags: "VISIBLE", parent: window, window: &data.window, language: "en_us")]
    #[nwg_layout_item(layout: grid, row: 0, col: 1)]
    webview_container: WebviewContainer,

    #[nwg_layout(margin: [0; 4], parent: frame, max_row: Some(6), max_column: Some(1), spacing: 0)]
    #[nwg_layout_item(layout: grid, row: 0, col: 0)]
    grid2: GridLayout,

    #[nwg_control(size: (600, 800) , parent: window)]
    frame: Frame,

    #[nwg_control(text: "hello")]
    #[nwg_layout_item(layout: grid2, row: 0, col: 0,size: (100, 25))]
    #[nwg_events( OnButtonClick: [DemoUi::say_hello] )]
    hello_button: nwg::Button,

    #[nwg_control(text: "hello", size: (600,1200))]
    #[nwg_layout_item(layout: grid2, row: 1, col: 0)]
    // #[nwg_events( OnButtonClick: [DemoUi::add_item] )]
    rich_text_box: nwg::RichTextBox,

    buttons: RefCell<Vec<nwg::Button>>,
    handlers: RefCell<Vec<nwg::EventHandler>>,
}

impl DemoUi {
    const SIZE: (i32, i32) = (1920, 1080);
    fn say_hello(&self) {
        // let (_env, _contrller, webview) = self.webview_container.ready_block().unwrap();
        // // webview.open_dev_tools_window();
        //
        // let _ = webview.execute_script("document.cookie", move |js_cookie| {
        //     nwg::simple_message("title", &js_cookie);
        //     fetch(js_cookie);
        //     Ok(())
        // });
        let cookie = fs::read_to_string(COOKIE_FILE).unwrap();
        self.rich_text_box.set_text(&cookie);
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
    pub fn executor(&self) -> Result<LocalPool, Box<dyn Error>> {
        let executor = LocalPool::new();
        let webview_ready_fut = self.webview_container.ready_fut()?;

        executor.spawner().spawn_local(
            async move {
                let (_, _, webview) = webview_ready_fut.await;

                let _ = webview.add_web_resource_requested_filter(
                    FILTER_URL,
                    webview2_sys::WebResourceContext::All,
                );
                // let _ = webview.add_web_resource_requested_filter(
                //     "*video.twimg.com/amplify_video*",
                //     webview2_sys::WebResourceContext::All,
                // );

                let _ = webview.add_web_resource_requested(move|_wb, args| {
                    if let Ok(request) = args.get_request() {
                        if let Ok(headers) = request.get_headers() {
                            let uri = request
                                .get_uri()
                                .unwrap();

                            if uri.contains(COOKIE_URL)
                            {
                                let mut map: HashMap<String, String> = HashMap::new();
                                for (k, v) in headers.get_iterator().unwrap() {
                                    map.insert(k, v);
                                }
                                let json = serde_json::to_string_pretty(&map).unwrap();
                                if !Path::new(COOKIE_FILE).exists() {

                                    println!("Cookie:{}", &json);
                                    let _ = std::fs::write(COOKIE_FILE, json.as_bytes());
                                }
                            }
                            if uri.ends_with(".ts") {
                                let vec = uri.rsplit("/").collect::<Vec<&str>>();
                                let filename = vec.iter().next().unwrap();
                                let mut file = OpenOptions::new()
                                    .read(true)
                                    .write(true)
                                    .append(true)
                                    .create(true)
                                    .open("video.txt")
                                    .unwrap();
                                file.write_all((uri.to_string() + "\n").as_bytes()).expect("write fail");
                                drop(file);
                                println!("uri:{uri}");
                            }

                            // if uri.contains("amplify_video") {
                            //     let headers1 = request.get_headers().unwrap();
                            //     println!(" amplify_video uri:{}", &uri);
                            //     for (k, v) in headers1.get_iterator().unwrap() {
                            //         println!(" amplify_video header:{},{}", &k,&v);
                            //     }
                            // }

                        }
                    }
                    if let Ok(response) = args.get_response() {
                        println!("response status:{}", response.get_status_code().unwrap());
                        // let Ok(content) = response.get_content(){
                        //     content.bytes().
                        // }
                    }

                    Ok(())
                });

                webview.navigate(HOME_URL)?;
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

// use weibo::weibo;

// static HOME_URL: &'static str = "https://weibo.com";
// static FILTER_URL: &'static str = "https://weibo.com/*";
// static COOKIE_URL: &'static str = "ajax/feed/groupstimeline";
// const COOKIE_FILE: &'static str = "weibo_cookie.json";

static HOME_URL: &'static str = "https://twitter.com/superhu8686";
static FILTER_URL: &'static str = "https://video.twimg.com/ext_tw_video/*";
static COOKIE_URL: &'static str = "/ext_tw_video/";
const COOKIE_FILE: &'static str = "twitter_cookie.json";
fn main() -> Result<(), Box<dyn Error>> {
    nwg::init()?;

    // 主窗体
    let  demo_ui_app = DemoUi::build_ui(Default::default())?;
    // 业务处理逻辑
    let mut executor = demo_ui_app.executor()?;
    // 阻塞主线程，等待用户手动关闭主窗体
    nwg::dispatch_thread_events_with_callback(move ||
        // 以 win32 UI 的事件循环为【反应器】，对接 futures crate 的【执行器】
        executor.run_until_stalled());
    Ok(())
}
