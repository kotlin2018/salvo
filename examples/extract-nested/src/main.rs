use salvo::macros::Extractible;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

#[fn_handler]
async fn show(req: &mut Request, res: &mut Response) {
    let content = format!(
        r#"<!DOCTYPE html>
    <html>
        <head>
            <title>Parse data</title>
        </head>
        <body>
            <h1>Hello, fill your profile</h1>
            <div id="result"></div>
            <form id="form" method="post">
                <label>First Name:</label><input type="text" name="first_name" />
                <label>Last Name:</label><input type="text" name="last_name" />
                <legend>What is Your Favorite Pet?</legend>      
                <input type="checkbox" name="lovers" value="Cats">Cats<br>      
                <input type="checkbox" name="lovers" value="Dogs">Dogs<br>      
                <input type="checkbox" name="lovers" value="Birds">Birds<br>    
                <input type="submit" value="Submit" />
            </form>
            <script> 
            let form = document.getElementById("form");
            form.addEventListener("submit", async (e) => {{
                e.preventDefault();
                let response = await fetch('/{}?username=jobs', {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/json',
                    }},
                    body: JSON.stringify({{
                        first_name: form.querySelector("input[name='first_name']").value,
                        last_name: form.querySelector("input[name='last_name']").value,
                        lovers: Array.from(form.querySelectorAll("input[name='lovers']:checked")).map(el => el.value),
                    }}),
                }});
                let text = await response.text();
                document.getElementById("result").innerHTML = text;
            }});
            </script>
        </body>
    </html>
    "#,
        req.params().get("id").unwrap()
    );
    res.render(Text::Html(content));
}
#[fn_handler]
async fn edit<'a>(good_man: GoodMan<'a>, res: &mut Response) {
    res.render(Json(good_man));
}

#[derive(Serialize, Deserialize, Extractible, Debug)]
#[extract(default_source(from = "body", format = "json"))]
struct GoodMan<'a> {
    // 这个宏表示 id 就是 url 中的参数，例如: http://localhost:7878/95，id 就等于 95
    #[extract(source(from = "param"))]
    id: i64,
    // 这个宏表示 username 就是 query 查询参数 (Query Params)
    #[extract(source(from = "query"))]
    username: &'a str,
    // 下面这几个没被宏修饰的，默认传参就是 application/json (format = json )
    first_name: String,
    last_name: String,
    lovers: Vec<String>,
    // 这个宏就是将 请求参数全部解析到结构体
    #[extract(source(from = "request"))]
    nested: Nested<'a>,
}

#[derive(Serialize, Deserialize, Extractible, Debug)]
#[extract(default_source(from = "body", format = "json"))]
struct Nested<'a> {
    #[extract(source(from = "param"))]
    id: i64,
    #[extract(source(from = "query"))]
    username: &'a str,
    first_name: String,
    last_name: String,
    #[extract(rename = "lovers")]
    #[serde(default)]
    pets: Vec<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let router = Router::with_path("<id>").get(show).post(edit);
    tracing::info!("Listening on http://127.0.0.1:7878");
    println!("Example url: http://127.0.0.1:7878/95");
    Server::new(TcpListener::bind("127.0.0.1:7878")).serve(router).await;
}
