use gtk::{prelude::*, Entry, ListBox};
use gtk::{Application, ApplicationWindow, Button};
use gtk4 as gtk;
use omic::message::Request;

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();

    let application = Application::builder()
        .application_id("com.omic.omic")
        .build();

    application.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("omic")
            .default_width(350)
            .default_height(350)
            .build();

        let content_box = ListBox::builder()
            .vexpand(true)
            .hexpand(true)
            .valign(gtk::Align::Fill)
            .halign(gtk::Align::Fill)
            .build();

        let address_text_box = Entry::new();
        let port_text_box = Entry::new();
        let button = Button::with_label("Connect");

        content_box.append(&address_text_box);
        content_box.append(&port_text_box);
        content_box.append(&button);

        button.connect_clicked(move |button| {
            // TODO: handle error with popup
            if button.label() == Some("Disconnect".into()) {
                omic::socket::Socket::create_request()
                    .request(Request::Disconnect)
                    .send()
                    .unwrap();
                button.set_label("Connect");
                return;
            }

            let address = address_text_box.text().as_str().to_owned();
            let port = port_text_box.text().as_str().to_owned();

            omic::socket::Socket::create_request()
                .request(Request::Connect { address, port })
                .send()
                .unwrap();
            button.set_label("Disconnect");
        });

        window.set_child(Some(&content_box));

        window.show();
    });

    application.run();

    Ok(())
}
