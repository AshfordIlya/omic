use gtk::{prelude::*, Entry, Label};
use gtk::{Application, ApplicationWindow, Box, Button};
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
            .default_height(150)
            .build();

        let box_ = Box::new(gtk::Orientation::Vertical, 8);

        let label = Label::new(Some("omic"));
        let address_text_box = Entry::new();
        let port_text_box = Entry::new();
        let button = Button::with_label("Connect");

        address_text_box.set_placeholder_text(Some("Enter IP Address"));
        port_text_box.set_placeholder_text(Some("Enter Port"));

        box_.append(&label);
        box_.append(&address_text_box);
        box_.append(&port_text_box);
        box_.append(&button);

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

        window.set_size_request(box_.width_request(), box_.height_request());
        window.set_child(Some(&box_));

        window.show();
    });

    application.run();

    Ok(())
}
