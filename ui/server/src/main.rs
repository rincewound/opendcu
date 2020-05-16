extern crate rouille;

use rouille::Response;

fn main() {
    println!("Now listening on localhost:8088");

    rouille::start_server("localhost:8088", move |request| {
        {
            let response = rouille::match_assets(&request, "../ui");

            if response.is_success() {
                return response;
            }
        }

        Response::html("404 error. Try <a href=\"/README.md\"`>README.md</a> or \
                        <a href=\"/src/lib.rs\">src/lib.rs</a> for example.")
            .with_status_code(404)
    });
}
