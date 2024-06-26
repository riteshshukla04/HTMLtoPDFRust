#[macro_use] extern crate rocket;

use rocket::http::ContentType;
use rocket::response::Responder;
use rocket::response::Response;
use rocket::tokio::fs::File;
use rocket::tokio::io::AsyncReadExt;
use std::process::{Command, Stdio};
use std::io::Write;
use tempfile::NamedTempFile;

#[post("/generate_pdf", data = "<html_content>")]
async fn generate_pdf(html_content: String) -> Result<PDFResponse, std::io::Error> {
    let temp_pdf_file = NamedTempFile::new()?;
    let pdf_file_path = temp_pdf_file.path().to_str().unwrap().to_string();

    let mut child = Command::new("wkhtmltopdf")
        .arg("-") // Read HTML content from stdin
        .arg(&pdf_file_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute wkhtmltopdf");

    if let Some(mut stdin) = child.stdin.take() {
        write!(stdin, "{}", html_content).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    if output.status.success() {
        let mut pdf_file = File::open(&pdf_file_path).await?;
        let mut pdf_content = Vec::new();
        pdf_file.read_to_end(&mut pdf_content).await?;

        Ok(PDFResponse(pdf_content))
    } else {
        eprintln!("wkhtmltopdf error: {:?}", String::from_utf8_lossy(&output.stderr));
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to generate PDF: {}", String::from_utf8_lossy(&output.stderr)),
        ))
    }
}

struct PDFResponse(Vec<u8>);

impl<'r> Responder<'r, 'static> for PDFResponse {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Response::build()
            .header(ContentType::PDF)
            .sized_body(self.0.len(), std::io::Cursor::new(self.0))
            .ok()
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![generate_pdf])
}
